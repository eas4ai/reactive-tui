//! The code review's native animation, editor and Markdown findings
//! (docs/spec/roadmap.md, review-native-findings), each through its
//! requirement's falsifier: the animation module (docs/spec/animation.md,
//! ANI) and the text systems (docs/spec/text.md, TXT). A test's name starts
//! with its requirement, which is how scripts/cairn/review_native.py picks
//! it; tests that need a module's private parts live in that module, named
//! the same way.

#[path = "review_native/animation.rs"]
mod animation;
#[path = "review_native/text.rs"]
mod text;
