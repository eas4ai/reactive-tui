//! Internal Reactive-TUI renderer, imported from SuprTUI.
//! Source revision and notices are recorded in ../UPSTREAM.md.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod ansi;
pub mod audio;
pub mod blit;
pub mod buffer;
pub mod clipboard;
pub mod layout;
pub mod link;
pub mod media;
pub mod render;
pub mod sys;
pub mod term;
pub mod term_embedded;
pub mod text;
pub mod uni;
