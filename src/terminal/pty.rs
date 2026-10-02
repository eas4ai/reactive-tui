//! Retained platform PTY API. Each child and its IO have one joined owner.
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::PseudoTerminal;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::PseudoTerminal;
#[cfg(test)]
mod tests;
pub(crate) const MAX_INPUT: usize = 64 * 1024;
pub(crate) const OUTPUT_CHUNK_SIZE: usize = 4096;
