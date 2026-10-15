use std::fs;
use std::path::{Path, PathBuf};
use lopdf::{Document, Object, Stream};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureProfile {
    pub full_name: String,
    pub initials: String,
    pub style: String, // "script_formal", "script_modern", "script_casual", "drawn", "image"
    pub signature_data: Option<String>, // Base64 data URL if drawn or image
    pub court_s_block: String, // e.g. "/s/ Joshua Abrams"
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureStampResult {
    pub ok: bool,
    pub never_files: bool,
    pub output_path: String,
    pub page_number: u32,
    pub stamped_text: String,
    pub message: String,
}

fn signature_store_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("JustLegal")
        .join("signature_profile.json")
}

pub fn load_signature_profile() -> Result<SignatureProfile, String> {
    let path = signature_store_path();
    if !path.is_file() {
        return Ok(SignatureProfile {
            full_name: String::new(),
            initials: String::new(),
            style: "script_formal".to_string(),
            signature_data: None,
            court_s_block: String::new(),
            updated_at: chrono::Local::now().to_rfc3339(),
        });
    }

    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

pub fn save_signature_profile(profile: &SignatureProfile) -> Result<(), String> {
    let path = signature_store_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_vec_pretty(profile).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())
}

/// Stamp a signature text block onto a specified page of a PDF using lopdf.
pub fn stamp_pdf(
    input_path: &Path,
    signature_text: &str,
    page_number: u32,
    x: f32,
    y: f32,
    output_path: &Path,
) -> Result<SignatureStampResult, String> {
    if !input_path.is_file() {
        return Err(format!("Input file not found: {}", input_path.display()));
    }

    let mut doc = Document::load(input_path).map_err(|e| format!("Failed to load PDF: {e}"))?;
    let pages = doc.get_pages();
    let total_pages = pages.len() as u32;

    let target_page_num = if page_number == 0 || page_number > total_pages {
        total_pages // Default to last page if 0 or out of bounds
    } else {
        page_number
    };

    let page_id = *pages
        .get(&target_page_num)
        .ok_or_else(|| format!("Page {target_page_num} not found in PDF"))?;

    // Create a standard Type1 font dictionary for Helvetica
    let mut font_dict = lopdf::Dictionary::new();
    font_dict.set("Type", Object::Name(b"Font".to_vec()));
    font_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
    font_dict.set("BaseFont", Object::Name(b"Helvetica-Bold".to_vec()));
    let font_id = doc.add_object(Object::Dictionary(font_dict));

    // Ensure Page has a Resources dictionary with our Font
    let res_id = match doc.get_object(page_id).and_then(|o| o.as_dict()).ok().and_then(|d| d.get(b"Resources").ok()) {
        Some(Object::Reference(id)) => *id,
        Some(Object::Dictionary(d)) => {
            let cloned = d.clone();
            let new_res_id = doc.add_object(Object::Dictionary(cloned));
            if let Ok(page_dict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
                page_dict.set("Resources", Object::Reference(new_res_id));
            }
            new_res_id
        }
        _ => {
            let new_res_id = doc.add_object(Object::Dictionary(lopdf::Dictionary::new()));
            if let Ok(page_dict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
                page_dict.set("Resources", Object::Reference(new_res_id));
            }
            new_res_id
        }
    };

    let font_res_id = match doc.get_object(res_id).and_then(|o| o.as_dict()).ok().and_then(|d| d.get(b"Font").ok()) {
        Some(Object::Reference(id)) => *id,
        Some(Object::Dictionary(d)) => {
            let cloned = d.clone();
            let new_fid = doc.add_object(Object::Dictionary(cloned));
            if let Ok(res_dict) = doc.get_object_mut(res_id).and_then(|o| o.as_dict_mut()) {
                res_dict.set("Font", Object::Reference(new_fid));
            }
            new_fid
        }
        _ => {
            let new_fid = doc.add_object(Object::Dictionary(lopdf::Dictionary::new()));
            if let Ok(res_dict) = doc.get_object_mut(res_id).and_then(|o| o.as_dict_mut()) {
                res_dict.set("Font", Object::Reference(new_fid));
            }
            new_fid
        }
    };

    if let Ok(fonts_dict) = doc.get_object_mut(font_res_id).and_then(|o| o.as_dict_mut()) {
        fonts_dict.set("SigFont", Object::Reference(font_id));
    }

    // Escape parenthesis in signature text for PDF literal string
    let escaped_text = signature_text
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)");

    let date_now = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Stream commands to draw signature box, text, and date
    let stamp_stream_content = format!(
        "q\n\
         0 0 0 rg\n\
         BT\n\
         /SigFont 13 Tf\n\
         {x} {y} Td\n\
         ({escaped_text}) Tj\n\
         0 -14 Td\n\
         /SigFont 9 Tf\n\
         (Electronically signed: {date_now}) Tj\n\
         ET\n\
         Q\n"
    );

    let stream_obj = Stream::new(lopdf::Dictionary::new(), stamp_stream_content.into_bytes());
    let stream_id = doc.add_object(Object::Stream(stream_obj));

    // Append this content stream to the page
    let page_dict = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()).unwrap();
    match page_dict.get_mut(b"Contents") {
        Ok(Object::Reference(content_ref)) => {
            let old_id = *content_ref;
            page_dict.set("Contents", Object::Array(vec![Object::Reference(old_id), Object::Reference(stream_id)]));
        }
        Ok(Object::Array(ref mut arr)) => {
            arr.push(Object::Reference(stream_id));
        }
        _ => {
            page_dict.set("Contents", Object::Reference(stream_id));
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    doc.save(output_path).map_err(|e| format!("Failed to save stamped PDF: {e}"))?;

    Ok(SignatureStampResult {
        ok: true,
        never_files: true,
        output_path: output_path.to_string_lossy().into_owned(),
        page_number: target_page_num,
        stamped_text: signature_text.to_string(),
        message: format!("Stamped signature onto page {} of {}", target_page_num, output_path.display()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_profile_default() {
        let p = SignatureProfile {
            full_name: "Test Signer".to_string(),
            initials: "TS".to_string(),
            style: "script_formal".to_string(),
            signature_data: None,
            court_s_block: "/s/ Test Signer".to_string(),
            updated_at: "2026-10-15".to_string(),
        };
        assert_eq!(p.court_s_block, "/s/ Test Signer");
    }
}

