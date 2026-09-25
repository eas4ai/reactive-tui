//! Owner-local targets for animations that start from presented property values.
use crate::{
    component::{bridge::element_style, Element},
    layout::style::StyleBuilder,
    reactive::wake::AppWaker,
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, Weak},
};

/// A current-value animation could not resolve its target or expression.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AnimationTargetError {
    #[error("animation target {0:?} is not present in its owner")]
    /// The owner or uniquely keyed element is no longer present.
    MissingTarget(String),
    #[error("animation target ID {0:?} is ambiguous in this owner")]
    /// More than one element in the owner has the requested key.
    AmbiguousTarget(String),
    #[error("current-value property {0:?} requires an App or ScreenManager target handle")]
    /// A current-value property was passed with only a string ID.
    HandleRequired(String),
    #[error("target has no declared numeric property {0:?}")]
    /// The target has no declared or representable value for this property.
    MissingProperty(String),
    #[error("invalid animation value for {0:?}: {1}")]
    /// An expression, endpoint or sample is invalid.
    InvalidValue(String, String),
    #[error("bound numeric targets do not support {0}; use explicit unbound animation values")]
    /// The bound numeric API does not implement this property family.
    UnsupportedProperty(&'static str),
    #[error("animation target state lock is poisoned")]
    /// A target state lock could not be acquired safely.
    Poisoned,
}

type Identity = (
    Vec<u64>,
    std::mem::Discriminant<crate::component::ElementType>,
);
fn identity(element: &Element) -> Identity {
    (
        element.metadata.component_instances.clone(),
        std::mem::discriminant(&element.element_type),
    )
}
#[derive(Clone, Debug)]
struct PresentedTarget {
    values: HashMap<String, f32>,
    identity: Identity,
}
#[derive(Debug)]
struct TargetState {
    live: bool,
    current: PresentedTarget,
    samples: HashMap<String, f32>,
}
struct TargetSlot {
    state: Mutex<TargetState>,
    wake: AppWaker,
}
impl std::fmt::Debug for TargetSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetSlot").finish_non_exhaustive()
    }
}

/// A weak reference to one target generation in one App or screen.
/// Removing the element invalidates this handle, even if its ID is reused later.
#[derive(Clone, Debug)]
pub struct AnimationTarget {
    id: String,
    slot: Weak<TargetSlot>,
}
impl AnimationTarget {
    /// The element key supplied by `.id()` or `.key()`.
    pub fn id(&self) -> &str {
        &self.id
    }
    fn slot(&self) -> Result<Arc<TargetSlot>, AnimationTargetError> {
        self.slot
            .upgrade()
            .ok_or_else(|| AnimationTargetError::MissingTarget(self.id.clone()))
    }
    pub(crate) fn current(&self) -> Result<HashMap<String, f32>, AnimationTargetError> {
        let slot = self.slot()?;
        let state = slot
            .state
            .lock()
            .map_err(|_| AnimationTargetError::Poisoned)?;
        if !state.live {
            return Err(AnimationTargetError::MissingTarget(self.id.clone()));
        }
        Ok(state.current.values.clone())
    }
    pub(crate) fn sample(&self, values: HashMap<String, f32>) -> Result<(), AnimationTargetError> {
        let slot = self.slot()?;
        let mut state = slot
            .state
            .lock()
            .map_err(|_| AnimationTargetError::Poisoned)?;
        if !state.live {
            return Err(AnimationTargetError::MissingTarget(self.id.clone()));
        }
        state.samples.extend(values);
        drop(state);
        slot.wake.request_redraw();
        Ok(())
    }
    /// Remove animation overrides so the element's authored properties paint again.
    pub fn clear(&self) -> Result<(), AnimationTargetError> {
        let slot = self.slot()?;
        let mut state = slot
            .state
            .lock()
            .map_err(|_| AnimationTargetError::Poisoned)?;
        if !state.live {
            return Err(AnimationTargetError::MissingTarget(self.id.clone()));
        }
        state.samples.clear();
        drop(state);
        slot.wake.request_redraw();
        Ok(())
    }
}

#[derive(Debug, Default)]
struct RegistryState {
    targets: HashMap<String, Arc<TargetSlot>>,
    ambiguous: HashSet<String>,
}
/// Cloneable lookup context. It does not keep the owning App or screen alive.
/// Obtain this before `App::run`, then look up targets after a frame is presented.
#[derive(Clone, Debug)]
pub struct AnimationTargetContext {
    state: Weak<Mutex<RegistryState>>,
}
impl AnimationTargetContext {
    /// Find one uniquely keyed element in the owner's last presented tree.
    pub fn target(&self, id: &str) -> Result<AnimationTarget, AnimationTargetError> {
        let owner = self
            .state
            .upgrade()
            .ok_or_else(|| AnimationTargetError::MissingTarget(id.into()))?;
        let state = owner.lock().map_err(|_| AnimationTargetError::Poisoned)?;
        if state.ambiguous.contains(id) {
            return Err(AnimationTargetError::AmbiguousTarget(id.into()));
        }
        let slot = state
            .targets
            .get(id)
            .ok_or_else(|| AnimationTargetError::MissingTarget(id.into()))?;
        Ok(AnimationTarget {
            id: id.into(),
            slot: Arc::downgrade(slot),
        })
    }
}

/// The targets one presented frame publishes, collected without publishing
/// them. App keeps the acknowledged frame's copy, so a present that reports
/// a flush failure republishes it without keeping that frame's tree, and the
/// handlers in it, alive (PIP-002).
#[derive(Clone, Debug, Default)]
pub(crate) struct PresentedTargets {
    values: HashMap<String, PresentedTarget>,
    ambiguous: HashSet<String>,
}

pub(crate) struct TargetRegistry {
    state: Arc<Mutex<RegistryState>>,
    wake: AppWaker,
}
impl TargetRegistry {
    pub(crate) fn new(wake: AppWaker) -> Self {
        Self {
            state: Arc::new(Mutex::new(RegistryState::default())),
            wake,
        }
    }
    pub(crate) fn context(&self) -> AnimationTargetContext {
        AnimationTargetContext {
            state: Arc::downgrade(&self.state),
        }
    }
    pub(crate) fn clear(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        for slot in state.targets.values() {
            slot.state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .live = false;
        }
        state.targets.clear();
        state.ambiguous.clear();
    }
    pub(crate) fn publish(
        &mut self,
        element: &Element,
        layouts: Option<&[crate::backend::PresentedLayout]>,
        offset: usize,
    ) -> crate::error::Result<()> {
        let targets = Self::collect(element, layouts, offset)?;
        self.publish_targets(targets).map(drop)
    }
    /// The targets `element` presents with `layouts`, numbering elements in
    /// preorder from `offset`.
    pub(crate) fn collect(
        element: &Element,
        layouts: Option<&[crate::backend::PresentedLayout]>,
        offset: usize,
    ) -> crate::error::Result<PresentedTargets> {
        fn visit(
            element: &Element,
            index: &mut usize,
            sizes: &HashMap<usize, (f32, f32)>,
            values: &mut HashMap<String, PresentedTarget>,
            ambiguous: &mut HashSet<String>,
        ) -> crate::error::Result<()> {
            let size = sizes.get(index);
            *index += 1;
            if let Some(id) = &element.key {
                let style = paint_style(element)?;
                let mut current = element.metadata.animation_values.clone();
                current.extend([
                    ("opacity".into(), style.opacity.unwrap_or(1.0)),
                    ("scaleX".into(), style.motion.transform.scale_x),
                    ("scaleY".into(), style.motion.transform.scale_y),
                    ("rotate".into(), style.motion.transform.rotation),
                ]);
                for (name, base, percent, dimension) in [
                    (
                        "translateX",
                        style.motion.transform.x,
                        style.motion.transform.x_percent,
                        size.map(|s| s.0),
                    ),
                    (
                        "translateY",
                        style.motion.transform.y,
                        style.motion.transform.y_percent,
                        size.map(|s| s.1),
                    ),
                ] {
                    current.remove(name);
                    if percent == 0.0 {
                        current.insert(name.into(), base);
                    } else if let Some(dimension) = dimension {
                        current.insert(name.into(), base + percent * dimension);
                    }
                }
                current.remove("scale");
                if style.motion.transform.scale_x == style.motion.transform.scale_y {
                    current.insert("scale".into(), style.motion.transform.scale_x);
                }
                if values
                    .insert(
                        id.clone(),
                        PresentedTarget {
                            values: current,
                            identity: identity(element),
                        },
                    )
                    .is_some()
                {
                    ambiguous.insert(id.clone());
                }
            }
            for child in &element.children {
                visit(child, index, sizes, values, ambiguous)?;
            }
            Ok(())
        }
        let mut values = HashMap::new();
        let mut ambiguous = HashSet::new();
        let sizes = layouts
            .unwrap_or_default()
            .iter()
            .map(|layout| (layout.element_index, layout.layout.size))
            .collect();
        let mut index = offset;
        visit(element, &mut index, &sizes, &mut values, &mut ambiguous)?;
        Ok(PresentedTargets { values, ambiguous })
    }
    /// Publish targets collected from a presented frame and return the ones
    /// they replace, which are those of the frame published before it. The
    /// replaced values move out of their slots, so keeping them costs no
    /// copy on the frame path.
    pub(crate) fn publish_targets(
        &mut self,
        targets: PresentedTargets,
    ) -> crate::error::Result<PresentedTargets> {
        let PresentedTargets { values, ambiguous } = targets;
        let mut state = self.state.lock().map_err(|_| {
            crate::error::ReactiveError::invalid_state("animation target registry lock poisoned")
        })?;
        let mut replaced = HashMap::new();
        state.targets.retain(|id, slot| {
            let mut slot = slot.state.lock().unwrap_or_else(|error| error.into_inner());
            let keep = values
                .get(id)
                .is_some_and(|value| value.identity == slot.current.identity)
                && !ambiguous.contains(id);
            if !keep {
                slot.live = false;
                replaced.insert(id.clone(), slot.current.clone());
            }
            keep
        });
        for (id, current) in values {
            if ambiguous.contains(&id) {
                continue;
            }
            if let Some(slot) = state.targets.get(&id) {
                let previous = std::mem::replace(
                    &mut slot
                        .state
                        .lock()
                        .map_err(|_| {
                            crate::error::ReactiveError::invalid_state(
                                "animation target lock poisoned",
                            )
                        })?
                        .current,
                    current,
                );
                replaced.insert(id, previous);
            } else {
                let slot = Arc::new(TargetSlot {
                    state: Mutex::new(TargetState {
                        live: true,
                        current,
                        samples: HashMap::new(),
                    }),
                    wake: self.wake.clone(),
                });
                state.targets.insert(id, slot);
            }
        }
        Ok(PresentedTargets {
            values: replaced,
            ambiguous: std::mem::replace(&mut state.ambiguous, ambiguous),
        })
    }
    pub(crate) fn apply(&self, element: &mut Element) -> crate::error::Result<()> {
        let samples = {
            let state = self.state.lock().map_err(|_| {
                crate::error::ReactiveError::invalid_state(
                    "animation target registry lock poisoned",
                )
            })?;
            state
                .targets
                .iter()
                .map(|(id, slot)| {
                    let slot = slot.state.lock().map_err(|_| {
                        crate::error::ReactiveError::invalid_state("animation target lock poisoned")
                    })?;
                    Ok((
                        id.clone(),
                        (slot.current.identity.clone(), slot.samples.clone()),
                    ))
                })
                .collect::<crate::error::Result<HashMap<_, _>>>()?
        };
        fn apply(
            element: &mut Element,
            samples: &HashMap<String, (Identity, HashMap<String, f32>)>,
            counts: &HashMap<String, usize>,
        ) -> crate::error::Result<()> {
            if let Some((_, values)) = element
                .key
                .as_ref()
                .filter(|id| counts.get(*id) == Some(&1))
                .and_then(|id| samples.get(id))
                .filter(|(target, values)| *target == identity(element) && !values.is_empty())
            {
                let mut style = paint_style(element)?;
                // Uniform scale precedes explicit axis overrides, as in the property list.
                let mut values = values.iter().collect::<Vec<_>>();
                values.sort_by_key(|(name, _)| *name);
                for (name, value) in values {
                    match name.as_str() {
                        "opacity" => style = style.opacity(*value),
                        "translateX" => {
                            style.motion.transform.x = *value;
                            style.motion.transform.x_percent = 0.0;
                        }
                        "translateY" => {
                            style.motion.transform.y = *value;
                            style.motion.transform.y_percent = 0.0;
                        }
                        "scaleX" => style.motion.transform.scale_x = *value,
                        "scaleY" => style.motion.transform.scale_y = *value,
                        "scale" => {
                            style.motion.transform.scale_x = *value;
                            style.motion.transform.scale_y = *value;
                        }
                        "rotate" => style.motion.transform.rotation = *value,
                        _ => {
                            element
                                .metadata
                                .animation_values
                                .insert(name.clone(), *value);
                        }
                    }
                }
                element.metadata.paint_style = Some(Arc::new(style.snapshot()));
            }
            for child in &mut element.children {
                apply(child, samples, counts)?;
            }
            Ok(())
        }
        fn count(element: &Element, counts: &mut HashMap<String, usize>) {
            if let Some(id) = &element.key {
                *counts.entry(id.clone()).or_default() += 1;
            }
            for child in &element.children {
                count(child, counts);
            }
        }
        let mut counts = HashMap::new();
        count(element, &mut counts);
        apply(element, &samples, &counts)
    }
}
impl Drop for TargetRegistry {
    fn drop(&mut self) {
        self.clear();
    }
}
fn paint_style(element: &Element) -> crate::error::Result<StyleBuilder> {
    match &element.metadata.paint_style {
        Some(style) => style.restore(),
        None => element_style(element),
    }
}
