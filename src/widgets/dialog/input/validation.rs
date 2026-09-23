use super::*;
use lru::LruCache;
use std::{
    num::NonZeroUsize,
    sync::{Mutex, OnceLock},
};

fn pattern_matches(pattern: &str, value: &str) -> Result<bool, String> {
    type Compiled = Result<regex::Regex, String>;
    static CACHE: OnceLock<Mutex<LruCache<String, Compiled>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(LruCache::new(NonZeroUsize::new(64).unwrap())));
    let cached = cache.lock().unwrap().get(pattern).cloned();
    let compiled = cached.unwrap_or_else(|| {
        let result = regex::Regex::new(pattern)
            .map_err(|error| format!("Invalid validation pattern: {error}"));
        cache
            .lock()
            .unwrap()
            .put(pattern.to_string(), result.clone());
        result
    });
    compiled.map(|regex| regex.is_match(value))
}

fn matches_type(rule: &ValidationRuleType, value: &str) -> Result<bool, String> {
    Ok(match rule {
        ValidationRuleType::Required => !value.trim().is_empty(),
        ValidationRuleType::MinLength(length) => value.graphemes(true).count() >= *length,
        ValidationRuleType::MaxLength(length) => value.graphemes(true).count() <= *length,
        ValidationRuleType::Pattern(pattern) => return pattern_matches(pattern, value),
        ValidationRuleType::Email => return pattern_matches(r"^[^\s@]+@[^\s@]+\.[^\s@]+$", value),
        ValidationRuleType::Url => value.split_once("://").is_some_and(|(scheme, rest)| {
            matches!(scheme, "http" | "https")
                && !rest.split('/').next().unwrap_or("").is_empty()
                && !rest.chars().any(char::is_whitespace)
        }),
        ValidationRuleType::Number => value.parse::<f64>().is_ok_and(f64::is_finite),
        ValidationRuleType::Phone => value.chars().filter(char::is_ascii_digit).count() >= 10,
        ValidationRuleType::Custom(_) => true,
    })
}

fn mask_matches(mask: &str, value: &str) -> Result<bool, String> {
    let mut tokens = mask.graphemes(true);
    let mut input = value.graphemes(true);
    let mut matches = true;
    while let Some(token) = tokens.next() {
        let escaped = token == "\\";
        let token = if escaped {
            tokens
                .next()
                .ok_or("Input mask ends with an escape; add a literal after the backslash")?
        } else {
            token
        };
        if token.chars().any(char::is_control) {
            return Err("Input mask cannot contain control characters".into());
        }
        let grapheme = input.next();
        matches &= match (escaped, token, grapheme) {
            (false, "#", Some(value)) => value.len() == 1 && value.as_bytes()[0].is_ascii_digit(),
            (false, "A", Some(value)) => {
                pattern_matches(r"^\p{Alphabetic}[\p{Alphabetic}\p{Mark}]*$", value)?
            }
            (false, "*", Some(value)) => !value.chars().any(char::is_control),
            (_, literal, Some(value)) => literal == value,
            (_, _, None) => false,
        };
    }
    // Optional emptiness is valid, but a malformed configuration is always an error.
    Ok(value.is_empty() || (matches && input.next().is_none()))
}

pub(super) fn validate(options: &InputDialogOptions, value: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    if options.input.required && value.trim().is_empty() {
        errors.push("This field is required".to_string());
    }
    if options
        .input
        .max_length
        .is_some_and(|maximum| value.graphemes(true).count() > maximum)
    {
        errors.push("Input exceeds the maximum length".to_string());
    }
    if let Some(mask) = &options.input.mask {
        match mask_matches(mask, value) {
            Ok(false) => errors.push("Input does not match the format".into()),
            Err(error) => errors.push(error),
            Ok(true) => {}
        }
    }
    let rule = match options.input.input_type {
        InputType::Email => Some(ValidationRuleType::Email),
        InputType::Number => Some(ValidationRuleType::Number),
        InputType::Url => Some(ValidationRuleType::Url),
        InputType::Phone => Some(ValidationRuleType::Phone),
        _ => None,
    };
    if !value.is_empty() {
        if let Some(rule) = rule {
            match matches_type(&rule, value) {
                Ok(false) => errors.push(format!(
                    "Input does not match the {:?} format",
                    options.input.input_type
                )),
                Err(error) => errors.push(error),
                _ => {}
            }
        }
    }
    if let Some(validation) = &options.validation {
        for rule in &validation.rules {
            if value.is_empty() && !rule.required && rule.rule_type != ValidationRuleType::Required
            {
                continue;
            }
            match matches_type(&rule.rule_type, value) {
                Ok(false) => errors.push(rule.message.clone()),
                Err(error) => errors.push(error),
                _ => {}
            }
            if matches!(rule.rule_type, ValidationRuleType::Custom(_))
                && validation.custom_validator.is_none()
                && options.on_validate.is_none()
            {
                errors.push("Custom validation requires a validator callback".to_string());
            }
        }
        if let Some(callback) = &validation.custom_validator {
            let result = callback(value);
            if !result.valid {
                errors.push(
                    result
                        .message
                        .unwrap_or_else(|| "Input is invalid".to_string()),
                );
            }
            warnings.extend(result.warnings);
        }
    }
    if let Some(callback) = &options.on_validate {
        let result = callback(value);
        if !result.valid {
            errors.push(
                result
                    .message
                    .unwrap_or_else(|| "Input is invalid".to_string()),
            );
        }
        warnings.extend(result.warnings);
    }
    ValidationResult {
        valid: errors.is_empty(),
        message: (!errors.is_empty()).then(|| errors.join("; ")),
        warnings,
    }
}
