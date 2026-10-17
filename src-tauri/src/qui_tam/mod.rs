//! Qui Tam / False Claims Act (31 U.S.C. § 3729) & EO 14395 Fraud Elimination Module.

pub mod types;
pub mod nppes;
pub mod rule9b;
pub mod observation;
pub mod records_pack;
pub mod disclosure;

#[cfg(test)]
mod tests;

pub use types::*;
pub use nppes::{lookup_nppes, verify_entity};
pub use rule9b::audit_case_rule9b;
pub use observation::ingest_observation;
pub use records_pack::generate_records_pack;
pub use disclosure::export_doj_disclosure;
