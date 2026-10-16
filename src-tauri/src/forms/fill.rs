//! Court Form / AcroForm Inspection and Auto-Fill Engine.

use std::fs;
use std::path::Path;
use crate::check::NEVER_FILE_NOTICE;
use super::types::{FormFieldInfo, FormFillResult, FormInspection};

pub fn inspect_court_form(pdf_path: &Path, case_dir: Option<&Path>) -> Result<FormInspection, String> {
    let fname = pdf_path.file_name().and_then(|s| s.to_str()).unwrap_or("form.pdf");
    let profile = case_dir.and_then(|d| crate::case_profile::load_case(d).ok());

    let form_number = if fname.contains("205") {
        Some("JDF 205 (Fee Waiver Motion)".to_string())
    } else if fname.contains("206") {
        Some("JDF 206 (Fee Waiver Order)".to_string())
    } else if fname.contains("1911") {
        Some("JDF 1911 (Notice of Appeal)".to_string())
    } else if fname.contains("1982") {
        Some("JDF 1982 IC (Motion for Extension of Time)".to_string())
    } else if fname.contains("1984") {
        Some("JDF 1984 IC (Response to Court Order)".to_string())
    } else if fname.contains("1986") {
        Some("JDF 1986 IC (Certificate of Service)".to_string())
    } else {
        None
    };

    let p = profile.as_ref();
    let petitioner = p.and_then(|cp| {
        cp.caption.appellants.first()
            .or_else(|| cp.caption.plaintiffs.first())
            .map(|pt| pt.name.clone())
    });
    let respondent = p.and_then(|cp| {
        cp.caption.appellees.first()
            .or_else(|| cp.caption.defendants.first())
            .map(|pt| pt.name.clone())
    });

    let fields = vec![
        FormFieldInfo {
            key: "court_name".to_string(),
            label: "Court or Agency Name".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.court.name.clone()),
            required: true,
        },
        FormFieldInfo {
            key: "county".to_string(),
            label: "County / Division".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.and_then(|cp| cp.court.division.clone()),
            required: false,
        },
        FormFieldInfo {
            key: "docket_number".to_string(),
            label: "Case / Docket Number".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.docket_number.clone()),
            required: true,
        },
        FormFieldInfo {
            key: "petitioner".to_string(),
            label: "Petitioner / Appellant / Plaintiff".to_string(),
            field_type: "text".to_string(),
            value_from_case: petitioner,
            required: true,
        },
        FormFieldInfo {
            key: "respondent".to_string(),
            label: "Respondent / Appellee / Defendant".to_string(),
            field_type: "text".to_string(),
            value_from_case: respondent,
            required: true,
        },
        FormFieldInfo {
            key: "filer_name".to_string(),
            label: "Filer Full Name".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.filer.name.clone()),
            required: true,
        },
        FormFieldInfo {
            key: "filer_phone".to_string(),
            label: "Filer Phone Number".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.filer.phone.clone()),
            required: false,
        },
        FormFieldInfo {
            key: "filer_email".to_string(),
            label: "Filer Email".to_string(),
            field_type: "text".to_string(),
            value_from_case: p.map(|cp| cp.filer.email.clone()),
            required: false,
        },
        FormFieldInfo {
            key: "signature".to_string(),
            label: "Electronic Signature".to_string(),
            field_type: "signature".to_string(),
            value_from_case: p.map(|cp| format!("/s/ {}", cp.filer.signature_name)),
            required: true,
        },
        FormFieldInfo {
            key: "date".to_string(),
            label: "Filing / Signing Date".to_string(),
            field_type: "date".to_string(),
            value_from_case: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
            required: true,
        },
        FormFieldInfo {
            key: "user_text".to_string(),
            label: "Grounds / Statement of Appeal / Relief Requested".to_string(),
            field_type: "textarea".to_string(),
            value_from_case: p.map(|cp| cp.notice_of_appeal.user_text.clone()),
            required: false,
        },
    ];

    Ok(FormInspection {
        form_name: fname.to_string(),
        form_path: pdf_path.display().to_string(),
        detected_form_number: form_number,
        fields,
    })
}

pub fn fill_court_form(
    pdf_path: &Path,
    case_dir: &Path,
    out_pdf: Option<&Path>,
    custom_values: Option<std::collections::HashMap<String, String>>,
) -> Result<FormFillResult, String> {
    if crate::seal::is_dir_sealed(case_dir) {
        return Err("This case is sealed. Form fill export from the Documents pointer is refused.".into());
    }
    let profile = crate::case_profile::load_case(case_dir)?;
    let inspection = inspect_court_form(pdf_path, Some(case_dir))?;

    let mut values_map = std::collections::HashMap::new();
    for f in &inspection.fields {
        if let Some(v) = &f.value_from_case {
            values_map.insert(f.key.clone(), v.clone());
        }
    }
    if let Some(custom) = custom_values {
        for (k, v) in custom {
            values_map.insert(k, v);
        }
    }

    let fname = pdf_path.file_stem().and_then(|s| s.to_str()).unwrap_or("form");
    let exports_dir = case_dir.join("exports");
    fs::create_dir_all(&exports_dir).map_err(|e| e.to_string())?;

    let target_pdf = out_pdf
        .map(Path::to_path_buf)
        .unwrap_or_else(|| exports_dir.join(format!("filled-{}.pdf", fname)));
    let target_json = exports_dir.join(format!("filled-{}.json", fname));

    // Save JSON data companion
    let json_data = serde_json::to_string_pretty(&values_map).map_err(|e| e.to_string())?;
    fs::write(&target_json, json_data).map_err(|e| e.to_string())?;

    // Generate court-shaped PDF with Typst
    let court_name = values_map.get("court_name").cloned().unwrap_or(profile.court.name);
    let docket = values_map.get("docket_number").cloned().unwrap_or(profile.docket_number);
    let pet = values_map.get("petitioner").cloned().unwrap_or_else(|| "Petitioner".into());
    let resp = values_map.get("respondent").cloned().unwrap_or_else(|| "Respondent".into());
    let sig = values_map.get("signature").cloned().unwrap_or_else(|| format!("/s/ {}", profile.filer.signature_name));
    let fname_val = values_map.get("filer_name").cloned().unwrap_or(profile.filer.name);
    let phone_val = values_map.get("filer_phone").cloned().unwrap_or_default();
    let email_val = values_map.get("filer_email").cloned().unwrap_or_default();
    let date_val = values_map.get("date").cloned().unwrap_or_else(|| chrono::Local::now().format("%B %d, %Y").to_string());
    let body_text = values_map.get("user_text").cloned().unwrap_or_default();

    let title_form = inspection.detected_form_number.unwrap_or_else(|| format!("OFFICIAL COURT FORM: {}", fname));

    let typst_content = format!(
r##"
#set page(paper: "us-letter", margin: (x: 1in, y: 1in))
#set text(font: "Times New Roman", size: 12pt)
#set par(justify: false, leading: 0.8em)

#align(center)[
  #strong("{court_name}") \
  #v(0.3em)
  Docket / Case No. {docket} \
  #v(1em)
  #strong([COURT-FILLED FORM ADDENDUM]) \
  #text(size: 11pt, style: "italic")[{title_form}]
]

#v(1.5em)

#table(
  columns: (1fr, 1fr),
  inset: 8pt,
  [ *Petitioner / Appellant:* \ {pet} ],
  [ *Respondent / Appellee:* \ {resp} ]
)

#v(1.5em)

#block(stroke: 0.5pt, inset: 10pt, width: 100%)[
  *Relief Requested / Statement:* \
  #v(0.5em)
  {body_text}
]

#v(2em)

*Filer Information & Verification:* \
Name: {fname_val} \
Phone: {phone_val} \
Email: {email_val} \
Date: {date_val} \
Signature: {sig} \

#v(2em)
#align(center)[
  #text(size: 8pt, fill: luma(100))[
    {REVIEW_NOTICE}
  ]
]
"##,
        court_name = court_name,
        docket = docket,
        title_form = title_form,
        pet = pet,
        resp = resp,
        body_text = if body_text.is_empty() { "As stated in accompanying notice and pleadings." } else { &body_text },
        fname_val = fname_val,
        phone_val = phone_val,
        email_val = email_val,
        date_val = date_val,
        sig = sig,
        REVIEW_NOTICE = crate::case_profile::REVIEW_NOTICE
    );

    let pdf_bytes = crate::pdf::compile_typst(typst_content)?;
    fs::write(&target_pdf, pdf_bytes).map_err(|e| e.to_string())?;

    let mapped_count = values_map.len();

    Ok(FormFillResult {
        ok: true,
        never_files: true,
        form_path: pdf_path.display().to_string(),
        output_pdf_path: target_pdf.display().to_string(),
        output_json_path: target_json.display().to_string(),
        mapped_fields: mapped_count,
        notice: format!("{}\n{}", NEVER_FILE_NOTICE, crate::case_profile::REVIEW_NOTICE),
    })
}
