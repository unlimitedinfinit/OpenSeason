//! Installable forms catalogs and pack archives. Never bundles the 7 GB JustLegal tree.
//!
//! Submodules:
//! - `types`: Core data structures, options, and notices.
//! - `registry`: Catalog loaders, JustLegal registry adapters, and hashing.
//! - `installer`: Pack installation, verification, and uninstallation.
//! - `fill`: Court form AcroForm inspection and auto-fill.
//! - `library`: High-level search, case recommendations, SHA-256 fetching, and case attachment.

pub mod fill;
pub mod installer;
pub mod library;
pub mod registry;
pub mod types;

#[cfg(test)]
mod tests;

pub use fill::*;
pub use installer::*;
pub use library::*;
pub use registry::*;
pub use types::*;
