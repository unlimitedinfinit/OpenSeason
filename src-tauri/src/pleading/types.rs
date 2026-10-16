use crate::case_profile::CaseProfile;
use crate::legal_os;

pub const ENTER_ORDER_DATE_PROMPT: &str = legal_os::ENTER_ORDER_DATE_PROMPT;
pub const DATE_BLANK: &str = "Date: ____________";
pub const CERTIFICATE_BLANK: &str = "____________";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionStyle {
    Boxed,
    Classic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionVenue {
    NewJersey,
    Colorado,
    Generic,
}

impl CaptionVenue {
    pub fn as_str(self) -> &'static str {
        match self {
            CaptionVenue::NewJersey => "nj-boxed",
            CaptionVenue::Colorado => "co-boxed",
            CaptionVenue::Generic => "boxed",
        }
    }
}

pub fn caption_venue(profile: &CaseProfile) -> CaptionVenue {
    let state = profile.governing_state.trim().to_ascii_uppercase();
    if state == "CO" || state == "COLORADO" {
        return CaptionVenue::Colorado;
    }
    if state == "NJ" || state == "NEW JERSEY" {
        return CaptionVenue::NewJersey;
    }
    let blob = format!("{} {}", profile.court.id, profile.court.name).to_ascii_lowercase();
    if blob.contains("colorado") || blob.contains("co-mandamus") || blob.contains("co-app") {
        return CaptionVenue::Colorado;
    }
    if blob.contains("new jersey") || blob.contains("nj-app") || blob.contains("nj-") {
        return CaptionVenue::NewJersey;
    }
    CaptionVenue::Generic
}

/// Omit a division line when it is empty or already named in the court title
/// (e.g. "Appellate Division" on "Superior Court of New Jersey, Appellate Division").
pub fn division_if_distinct(court_name: &str, division: &str) -> String {
    let d = division.trim();
    if d.is_empty() {
        return String::new();
    }
    let name = court_name.to_ascii_lowercase();
    let div = d.to_ascii_lowercase();
    if name.contains(&div) {
        return String::new();
    }
    d.to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyBlock {
    Heading(String),
    Paragraph(String),
    Blank,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CourtDocument {
    pub style: CaptionStyle,
    pub court_name: String,
    pub court_division: String,
    pub court_address: Vec<String>,
    pub left_label: String,
    pub left_names: String,
    pub right_label: String,
    pub right_names: String,
    pub appellate_no: String,
    pub trial_no: String,
    pub division_courtroom: String,
    pub filer_name: String,
    pub filer_role: String,
    pub filer_address: Vec<String>,
    pub filer_email: String,
    pub filer_phone: String,
    pub title: String,
    pub intro: Vec<String>,
    pub body: Vec<BodyBlock>,
    pub signature_lines: Vec<String>,
    pub certification_heading: Option<String>,
    pub certification_text: Option<String>,
    pub certificate_heading: String,
    pub certificate_lines: Vec<String>,
    pub venue: CaptionVenue,
    pub lower_court_name: String,
}

pub fn caption_style(profile: &CaseProfile, kind: &str, override_layout: Option<&str>) -> CaptionStyle {
    let raw = override_layout
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(profile.caption_layout.trim());
    match raw.to_ascii_lowercase().as_str() {
        "classic" | "legacy" | "centered" => CaptionStyle::Classic,
        "boxed" | "nj" | "table" => CaptionStyle::Boxed,
        _ => match kind.trim().to_ascii_lowercase().as_str() {
            "notice_of_appeal" | "appeal" => CaptionStyle::Classic,
            _ => CaptionStyle::Boxed,
        },
    }
}

pub fn paper_noun(title: &str) -> &'static str {
    let t = title.to_ascii_lowercase();
    if t.contains("notice") {
        "notice"
    } else if t.contains("brief") {
        "brief"
    } else if t.contains("complaint") {
        "complaint"
    } else if t.contains("petition") {
        "petition"
    } else if t.contains("motion") {
        "motion"
    } else {
        "paper"
    }
}

pub fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn typst_escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('@', "\\@")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('<', "\\<")
        .replace('>', "\\>")
}
