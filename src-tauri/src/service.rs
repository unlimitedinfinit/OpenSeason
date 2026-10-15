use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::case_profile::{CaseProfile, REVIEW_NOTICE};
use crate::documents::typst_escape;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRecipient {
    pub name: String,
    pub role: String,
    pub address: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCertParams {
    pub filing_title: String,
    pub service_date: String,
    pub method: String,
    pub recipients: Vec<ServiceRecipient>,
    pub signature_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCertResult {
    pub ok: bool,
    pub never_files: bool,
    pub notice: String,
    pub cert_pdf: String,
    pub cert_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedOrderResult {
    pub ok: bool,
    pub never_files: bool,
    pub notice: String,
    pub order_pdf: String,
    pub order_docx: String,
}

const NEVER_FILE_NOTICE: &str =
    "This app never files or serves. The user signs, files, and serves by hand.";

pub fn generate_certificate_of_service(
    case_dir: &Path,
    profile: &CaseProfile,
    params: &ServiceCertParams,
) -> Result<ServiceCertResult, String> {
    let exports_dir = case_dir.join("exports");
    fs::create_dir_all(&exports_dir).map_err(|e| e.to_string())?;

    let left = if !profile.caption.appellants.is_empty() {
        &profile.caption.appellants
    } else {
        &profile.caption.plaintiffs
    };
    let right = if !profile.caption.appellees.is_empty() {
        &profile.caption.appellees
    } else {
        &profile.caption.defendants
    };

    let left_names = if left.is_empty() {
        "Plaintiff / Appellant".to_string()
    } else {
        left.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
    };
    let right_names = if right.is_empty() {
        "Defendant / Appellee".to_string()
    } else {
        right.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
    };

    let docket = if profile.docket_number.trim().is_empty() {
        "Docket No. _______________".to_string()
    } else {
        format!("Docket No. {}", profile.docket_number)
    };

    let filer_sig = if params.signature_name.trim().is_empty() {
        if !profile.filer.signature_name.trim().is_empty() {
            profile.filer.signature_name.clone()
        } else {
            profile.filer.name.clone()
        }
    } else {
        params.signature_name.clone()
    };

    let date_str = if params.service_date.trim().is_empty() {
        chrono::Local::now().format("%B %d, %Y").to_string()
    } else {
        params.service_date.clone()
    };

    let mut recipients_typst = String::new();
    let mut recipients_text = String::new();
    for r in &params.recipients {
        recipients_typst.push_str(&format!(
            "- *{}* ({})\n  Address: {}\n  Email: {}\n\n",
            typst_escape(&r.name),
            typst_escape(&r.role),
            typst_escape(if r.address.is_empty() { "On file" } else { &r.address }),
            typst_escape(if r.email.is_empty() { "None" } else { &r.email })
        ));
        recipients_text.push_str(&format!(
            "- {} ({})\n  Address: {}\n  Email: {}\n",
            r.name, r.role, r.address, r.email
        ));
    }

    let filing = if params.filing_title.trim().is_empty() {
        "the accompanying papers"
    } else {
        &params.filing_title
    };

    let typst_source = format!(
        r##"
#set page(paper: "us-letter", margin: (x: 1in, y: 1in), header: align(right)[
  #text(8pt, fill: rgb(120, 120, 120))[CERTIFICATE OF SERVICE --- {docket_header}]
])
#set text(font: "Times New Roman", size: 12pt)
#set par(justify: true, leading: 0.8em)

#align(center)[
  #strong[{court_name}] \
  {court_div}
]

#v(1em)

#grid(
  columns: (3.5in, 1fr),
  [
    *{left_names}*, \
    #v(0.3em)
    #h(1.5in) Plaintiff(s) / Appellant(s), \
    #v(0.5em)
    v. \
    #v(0.5em)
    *{right_names}*, \
    #v(0.3em)
    #h(1.5in) Defendant(s) / Appellee(s).
  ],
  [
    #box(stroke: (left: 1pt), inset: (left: 12pt))[
      *{docket}* \
      #v(0.5em)
      *CERTIFICATE OF SERVICE*
    ]
  ]
)

#v(2em)

#align(center)[*CERTIFICATE / PROOF OF SERVICE*]

#v(1em)

I hereby certify that on *{date_str}*, a true and correct copy of *{filing}* was served upon the following parties and counsel of record by *{method}*:

#v(1em)

{recipients_typst}

I certify under penalty of perjury under the laws of this jurisdiction that the foregoing statements made by me are true and correct.

#v(2.5em)

#grid(
  columns: (1fr, 1fr),
  [
    Dated: {date_str}
  ],
  [
    #line(length: 2.5in) \
    /s/ {filer_sig} \
    *{filer_name}* \
    {filer_role} \
    {filer_address} \
    Phone: {filer_phone} \
    Email: {filer_email}
  ]
)

#v(3em)
#line(length: 100%, stroke: 0.5pt + rgb(180, 180, 180))
#text(8pt, fill: rgb(120, 120, 120))[
  {never_file} \
  {review_notice}
]
"##,
        docket_header = typst_escape(if profile.docket_number.is_empty() { "UNFILED" } else { &profile.docket_number }),
        court_name = typst_escape(if profile.court.name.is_empty() { "COURT OF RECORD" } else { &profile.court.name }),
        court_div = typst_escape(profile.court.division.as_deref().unwrap_or("")),
        left_names = typst_escape(&left_names),
        right_names = typst_escape(&right_names),
        docket = typst_escape(&docket),
        date_str = typst_escape(&date_str),
        filing = typst_escape(filing),
        method = typst_escape(&params.method),
        recipients_typst = recipients_typst,
        filer_sig = typst_escape(&filer_sig),
        filer_name = typst_escape(if profile.filer.name.is_empty() { "Self-Represented Party" } else { &profile.filer.name }),
        filer_role = typst_escape(if profile.filer.role.is_empty() { "Pro Se Filer" } else { &profile.filer.role }),
        filer_address = typst_escape(&profile.filer.address_lines.join(", ")),
        filer_phone = typst_escape(if profile.filer.phone.is_empty() { "N/A" } else { &profile.filer.phone }),
        filer_email = typst_escape(if profile.filer.email.is_empty() { "N/A" } else { &profile.filer.email }),
        never_file = NEVER_FILE_NOTICE,
        review_notice = REVIEW_NOTICE,
    );

    let pdf_bytes = crate::pdf::compile_typst(typst_source)?;
    let out_pdf = exports_dir.join("certificate-of-service.pdf");
    fs::write(&out_pdf, &pdf_bytes).map_err(|e| e.to_string())?;

    let text_content = format!(
        "CERTIFICATE OF SERVICE\n\
         Case: {}\n\
         {}\n\n\
         On {}, I served {} via {}.\n\n\
         Recipients:\n{}\n\n\
         /s/ {}\n{}\n",
        profile.title,
        docket,
        date_str,
        filing,
        params.method,
        recipients_text,
        filer_sig,
        profile.filer.name
    );
    let out_txt = exports_dir.join("certificate-of-service.txt");
    let _ = fs::write(&out_txt, &text_content);

    Ok(ServiceCertResult {
        ok: true,
        never_files: true,
        notice: format!("{}\n{}", NEVER_FILE_NOTICE, REVIEW_NOTICE),
        cert_pdf: out_pdf.to_string_lossy().into_owned(),
        cert_text: text_content,
    })
}

pub fn generate_proposed_order(
    case_dir: &Path,
    profile: &CaseProfile,
    motion_title: &str,
    relief: &str,
) -> Result<ProposedOrderResult, String> {
    let exports_dir = case_dir.join("exports");
    fs::create_dir_all(&exports_dir).map_err(|e| e.to_string())?;

    let left = if !profile.caption.appellants.is_empty() {
        &profile.caption.appellants
    } else {
        &profile.caption.plaintiffs
    };
    let right = if !profile.caption.appellees.is_empty() {
        &profile.caption.appellees
    } else {
        &profile.caption.defendants
    };

    let left_names = if left.is_empty() {
        "Plaintiff / Appellant".to_string()
    } else {
        left.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
    };
    let right_names = if right.is_empty() {
        "Defendant / Appellee".to_string()
    } else {
        right.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ")
    };

    let docket = if profile.docket_number.trim().is_empty() {
        "Docket No. _______________".to_string()
    } else {
        format!("Docket No. {}", profile.docket_number)
    };

    let title_clean = if motion_title.trim().is_empty() {
        "RELIEF REQUESTED"
    } else {
        motion_title
    };

    let relief_text = if relief.trim().is_empty() {
        "1. The motion is GRANTED in its entirety;\n2. [Additional relief as ordered by the Court];\n3. A copy of this Order shall be served upon all parties within seven (7) days."
    } else {
        relief
    };

    let mut relief_typst = String::new();
    for line in relief_text.lines() {
        if !line.trim().is_empty() {
            relief_typst.push_str(&format!("  #v(0.5em)\n  {}\n", line));
        }
    }

    let typst_source = format!(
        r##"
#set page(paper: "us-letter", margin: (x: 1in, y: 1in))
#set text(font: "Times New Roman", size: 12pt)
#set par(justify: true, leading: 0.8em)

#align(center)[
  #strong[{court_name}] \
  {court_div}
]

#v(1em)

#grid(
  columns: (3.5in, 1fr),
  [
    *{left_names}*, \
    #v(0.3em)
    #h(1.5in) Plaintiff(s) / Appellant(s), \
    #v(0.5em)
    v. \
    #v(0.5em)
    *{right_names}*, \
    #v(0.3em)
    #h(1.5in) Defendant(s) / Appellee(s).
  ],
  [
    #box(stroke: (left: 1pt), inset: (left: 12pt))[
      *{docket}* \
      #v(0.5em)
      *CIVIL ACTION* \
      #v(0.5em)
      *ORDER*
    ]
  ]
)

#v(2em)

#align(center)[*ORDER GRANTING {title_clean}*]

#v(1.5em)

THIS MATTER having been opened to the Court upon application by *{filer_name}*, {filer_role}, seeking entry of an Order for *{title_clean}*; and the Court having considered the moving papers, and any opposition papers filed in response, and having heard oral argument if any, and for good cause shown;

#v(1em)

IT IS on this #line(length: 1.5in) day of #line(length: 1.8in), 20#line(length: 0.4in);

#v(1em)

#strong[ORDERED] that:

{relief_typst}

#v(4em)

#align(right)[
  #line(length: 3in) \
  #strong[JUDGE OF THE COURT]
]

#v(3em)
#line(length: 100%, stroke: 0.5pt + rgb(180, 180, 180))
#text(8pt, fill: rgb(120, 120, 120))[
  {never_file} \
  {review_notice}
]
"##,
        court_name = typst_escape(if profile.court.name.is_empty() { "COURT OF RECORD" } else { &profile.court.name }),
        court_div = typst_escape(profile.court.division.as_deref().unwrap_or("")),
        left_names = typst_escape(&left_names),
        right_names = typst_escape(&right_names),
        docket = typst_escape(&docket),
        title_clean = typst_escape(&title_clean.to_uppercase()),
        filer_name = typst_escape(if profile.filer.name.is_empty() { "Moving Party" } else { &profile.filer.name }),
        filer_role = typst_escape(if profile.filer.role.is_empty() { "pro se" } else { &profile.filer.role }),
        relief_typst = typst_escape(&relief_typst),
        never_file = NEVER_FILE_NOTICE,
        review_notice = REVIEW_NOTICE,
    );

    let pdf_bytes = crate::pdf::compile_typst(typst_source)?;
    let out_pdf = exports_dir.join("proposed-order.pdf");
    fs::write(&out_pdf, &pdf_bytes).map_err(|e| e.to_string())?;

    // Also write simple docx template or note
    let out_docx = exports_dir.join("proposed-order.docx");
    let _ = fs::write(&out_docx, b"Court Proposed Order");

    Ok(ProposedOrderResult {
        ok: true,
        never_files: true,
        notice: format!("{}\n{}", NEVER_FILE_NOTICE, REVIEW_NOTICE),
        order_pdf: out_pdf.to_string_lossy().into_owned(),
        order_docx: out_docx.to_string_lossy().into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case_profile::sample_appeal_profile;

    #[test]
    fn test_certificate_of_service_generates_pdf() {
        let temp_dir = std::env::temp_dir().join(format!("os-test-service-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        let profile = sample_appeal_profile();
        let params = ServiceCertParams {
            filing_title: "Motion for Reconsideration".to_string(),
            service_date: "January 16, 2026".to_string(),
            method: "eCourts Electronic Filing".to_string(),
            recipients: vec![ServiceRecipient {
                name: "Opposing Counsel".to_string(),
                role: "Counsel for Appellee".to_string(),
                address: "100 Court St, New Brunswick, NJ".to_string(),
                email: "counsel@example.com".to_string(),
            }],
            signature_name: "Jordan Example".to_string(),
        };

        let result = generate_certificate_of_service(&temp_dir, &profile, &params).unwrap();
        assert!(result.ok);
        assert!(result.never_files);
        assert!(Path::new(&result.cert_pdf).is_file());
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_proposed_order_generates_pdf() {
        let temp_dir = std::env::temp_dir().join(format!("os-test-order-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).unwrap();
        let profile = sample_appeal_profile();
        let result = generate_proposed_order(&temp_dir, &profile, "Motion for Extension of Time", "1. Time is extended by 30 days.").unwrap();
        assert!(result.ok);
        assert!(result.never_files);
        assert!(Path::new(&result.order_pdf).is_file());
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
