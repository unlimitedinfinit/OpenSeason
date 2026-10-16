//! Legal Operating System: Matter architecture, venue rules, procedural branching, and deadlines.

pub mod procedural;
pub mod types;
pub mod venue;

#[cfg(test)]
mod tests;

pub use procedural::*;
pub use types::*;
pub use venue::*;
