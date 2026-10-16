use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccommodationCategory {
    RemoteTelephonicAppearance,
    PhysicalMobilityAccess,
    VisualHearingAssistance,
    ExtendedReadingTime,
    SupportPersonAccompaniment,
    DocumentMagnificationAccessiblePdf,
    CommunicationInterpreter,
}

impl AccommodationCategory {
    pub fn label(&self) -> &'static str {
        match self {
            AccommodationCategory::RemoteTelephonicAppearance => "Remote Video / Telephonic Appearance (Virtual Hearing)",
            AccommodationCategory::PhysicalMobilityAccess => "Physical Mobility & Barrier-Free Access (Wheelchair, First Floor)",
            AccommodationCategory::VisualHearingAssistance => "Assistive Listening Device / CART Captioning",
            AccommodationCategory::ExtendedReadingTime => "Extended Time for Reading Filings / Frequent Breaks during Hearing",
            AccommodationCategory::SupportPersonAccompaniment => "Accompaniment by Non-Attorney Support Person / Caregiver",
            AccommodationCategory::DocumentMagnificationAccessiblePdf => "Accessible Digital PDF / Large Print Document Formats",
            AccommodationCategory::CommunicationInterpreter => "Qualified Sign Language or Specialized Communication Interpreter",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaRequestPayload {
    pub case_id: String,
    pub requester_name: String,
    pub requester_role: String, // "Plaintiff / Appellant (Self-Represented)"
    pub contact_phone: String,
    pub contact_email: String,
    pub accommodations: Vec<AccommodationCategory>,
    pub specific_needs_narrative: String,
    pub upcoming_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaPetitionReport {
    pub state: String,
    pub court_name: String,
    pub docket_number: String,
    pub ada_coordinator_title: String,
    pub statutory_authorities: Vec<String>,
    pub rendered_petition_text: String,
    pub submission_guidelines: String,
}
