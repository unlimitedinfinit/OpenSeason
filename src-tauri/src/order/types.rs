//! Types for order decoding, mail audit, and case bootstrap.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "detail")]
pub enum OrderSigner {
    ByTheCourt { division: String },
    NamedJudge { name: String, title: Option<String> },
    Clerk { name: Option<String> },
    Unknown,
}

impl Default for OrderSigner {
    fn default() -> Self {
        OrderSigner::Unknown
    }
}

impl OrderSigner {
    pub fn label(&self) -> String {
        match self {
            OrderSigner::ByTheCourt { division } => format!("BY THE COURT ({division})"),
            OrderSigner::NamedJudge { name, title } => match title {
                Some(t) => format!("{name}, {t}"),
                None => name.clone(),
            },
            OrderSigner::Clerk { name } => match name {
                Some(n) => format!("Clerk of Court ({n})"),
                None => "Clerk of Court".into(),
            },
            OrderSigner::Unknown => "Unknown / Unspecified Signer".into(),
        }
    }

    pub fn is_screening_division(&self) -> bool {
        matches!(self, OrderSigner::ByTheCourt { .. })
    }
}

/// The structured result of decoding a court order or notice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DecodedOrder {
    pub title: String,
    #[serde(default)]
    pub court_name: Option<String>,
    #[serde(default)]
    pub docket_number: Option<String>,
    #[serde(default)]
    pub order_date: Option<String>,
    pub signer: OrderSigner,
    pub is_boilerplate_notice: bool,
    #[serde(default)]
    pub operative_clauses: Vec<String>,
    #[serde(default)]
    pub candidate_response_window_days: Option<u32>,
    #[serde(default)]
    pub candidate_deadline: Option<String>,
    #[serde(default)]
    pub governing_rules_cited: Vec<String>,
    /// "verified" | "needs_verification"
    pub status: String,
    #[serde(default)]
    pub uncertainty_notes: Vec<String>,
    #[serde(default = "default_page_one")]
    pub source_page: u32,
    #[serde(default)]
    pub suggested_actions: Vec<String>,
}

fn default_page_one() -> u32 {
    1
}

/// Discrepancy analysis comparing mailing dates, clerk receipt timestamps, and deadlines.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MailAuditReport {
    pub case_id: String,
    pub order_entry_date: String,
    #[serde(default)]
    pub mailing_postmark: Option<String>,
    #[serde(default)]
    pub delivery_receipt_date: Option<String>,
    #[serde(default)]
    pub clerk_receipt_date: Option<String>,
    /// "physical_clerk_receipt" (e.g. Colorado) vs "mailbox_rule" (e.g. Federal pro se prisoner / service)
    pub jurisdiction_receipt_rule: String,
    pub mail_delay_days: i64,
    pub holiday_or_weekend_impact: bool,
    #[serde(default)]
    pub jurisdictional_traps: Vec<String>,
    #[serde(default)]
    pub recommended_actions: Vec<String>,
    pub status: String, // "flagged_risk" | "verified_timely" | "needs_verification"
}

/// Result of a case bootstrap / rescue operation on an unorganized folder.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BootstrapResult {
    pub case_id: String,
    pub case_path: String,
    pub title: String,
    pub files_scanned: usize,
    pub orders_imported: usize,
    pub filings_imported: usize,
    pub evidence_imported: usize,
    pub drafts_imported: usize,
    #[serde(default)]
    pub detected_docket_number: Option<String>,
    #[serde(default)]
    pub detected_court: Option<String>,
    #[serde(default)]
    pub pending_obligations: Vec<String>,
    pub summary: String,
}
