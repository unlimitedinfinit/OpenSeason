use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use lopdf::{Document, Object, ObjectId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfOperationResult {
    pub ok: bool,
    pub never_files: bool,
    pub operation: String,
    pub output_path: String,
    pub page_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactOperationResult {
    pub ok: bool,
    pub never_files: bool,
    pub output_path: String,
    pub redactions_count: usize,
    pub message: String,
}

/// Merge multiple PDFs into a single unified court submission PDF.
pub fn merge_pdfs(input_paths: &[PathBuf], output_path: &Path) -> Result<PdfOperationResult, String> {
    if input_paths.is_empty() {
        return Err("No input PDF files provided for merge.".to_string());
    }

    let mut target_doc = Document::with_version("1.7");
    let mut pages_obj: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut page_kids: Vec<Object> = Vec::new();
    let mut total_page_count = 0usize;

    // Dedicated ObjectId for the Pages tree root
    let pages_root_id = (1000000u32, 0u16);

    for path in input_paths {
        if !path.is_file() {
            return Err(format!("Input file not found: {}", path.display()));
        }
        let mut doc = Document::load(path).map_err(|e| format!("Failed to load {}: {e}", path.display()))?;
        doc.renumber_objects_with(target_doc.max_id + 1);

        let doc_pages = doc.get_pages();
        for (&_page_num, &page_id) in &doc_pages {
            total_page_count += 1;
            page_kids.push(Object::Reference(page_id));

            // Ensure the page references our new Pages root
            if let Ok(page_dict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
                page_dict.set("Parent", Object::Reference(pages_root_id));
            }
        }

        for (id, obj) in doc.objects {
            // Skip the old document catalog and pages root
            if let Ok(dict) = obj.as_dict() {
                if let Ok(type_name) = dict.get(b"Type").and_then(|o| o.as_name()) {
                    if type_name == b"Catalog" || type_name == b"Pages" {
                        continue;
                    }
                }
            }
            pages_obj.insert(id, obj);
        }

        target_doc.max_id = doc.max_id;
    }

    // Build the new Pages dictionary
    let mut pages_dict = lopdf::Dictionary::new();
    pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
    pages_dict.set("Count", Object::Integer(total_page_count as i64));
    pages_dict.set("Kids", Object::Array(page_kids));

    let pages_root_obj_id = target_doc.add_object(Object::Dictionary(pages_dict));

    // Build the new Catalog dictionary
    let mut catalog_dict = lopdf::Dictionary::new();
    catalog_dict.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog_dict.set("Pages", Object::Reference(pages_root_obj_id));

    let catalog_id = target_doc.add_object(Object::Dictionary(catalog_dict));
    target_doc.trailer.set("Root", Object::Reference(catalog_id));

    // Insert all collected page and resource objects
    for (id, mut obj) in pages_obj {
        // Point Parent to new Pages root
        if let Ok(dict) = obj.as_dict_mut() {
            if let Ok(type_name) = dict.get(b"Type").and_then(|o| o.as_name()) {
                if type_name == b"Page" {
                    dict.set("Parent", Object::Reference(pages_root_obj_id));
                }
            }
        }
        target_doc.objects.insert(id, obj);
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    target_doc.save(output_path).map_err(|e| format!("Failed to save merged PDF: {e}"))?;

    Ok(PdfOperationResult {
        ok: true,
        never_files: true,
        operation: "merge".to_string(),
        output_path: output_path.to_string_lossy().into_owned(),
        page_count: total_page_count,
        message: format!("Successfully merged {} files ({} pages total).", input_paths.len(), total_page_count),
    })
}

/// Split / Extract specific 1-based page numbers from a PDF.
pub fn split_pdf(input_path: &Path, pages_to_keep: &[u32], output_path: &Path) -> Result<PdfOperationResult, String> {
    if !input_path.is_file() {
        return Err(format!("Input file not found: {}", input_path.display()));
    }
    if pages_to_keep.is_empty() {
        return Err("No page numbers specified for extraction.".to_string());
    }

    let mut doc = Document::load(input_path).map_err(|e| format!("Failed to load PDF: {e}"))?;
    let all_pages = doc.get_pages();
    let total_original = all_pages.len();

    // Identify which page numbers to delete (all pages not in pages_to_keep)
    let pages_to_delete: Vec<u32> = all_pages
        .keys()
        .filter(|p| !pages_to_keep.contains(p))
        .cloned()
        .collect();

    doc.delete_pages(&pages_to_delete);
    doc.prune_objects();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    doc.save(output_path).map_err(|e| format!("Failed to save extracted PDF: {e}"))?;

    let kept_count = total_original.saturating_sub(pages_to_delete.len());

    Ok(PdfOperationResult {
        ok: true,
        never_files: true,
        operation: "split".to_string(),
        output_path: output_path.to_string_lossy().into_owned(),
        page_count: kept_count,
        message: format!("Extracted {} pages into {}.", kept_count, output_path.display()),
    })
}

/// Rotate pages by 90, 180, or 270 degrees clockwise.
pub fn rotate_pdf(
    input_path: &Path,
    degrees: i64,
    page_numbers: Option<Vec<u32>>,
    output_path: &Path,
) -> Result<PdfOperationResult, String> {
    if !input_path.is_file() {
        return Err(format!("Input file not found: {}", input_path.display()));
    }

    let mut doc = Document::load(input_path).map_err(|e| format!("Failed to load PDF: {e}"))?;
    let all_pages = doc.get_pages();
    let total_pages = all_pages.len();

    let target_pages: Vec<u32> = match page_numbers {
        Some(list) if !list.is_empty() => list,
        _ => all_pages.keys().cloned().collect(),
    };

    let mut rotated_count = 0;
    for page_num in &target_pages {
        if let Some(&page_id) = all_pages.get(page_num) {
            if let Ok(page_dict) = doc.get_object_mut(page_id).and_then(|o| o.as_dict_mut()) {
                let current_rot = page_dict
                    .get(b"Rotate")
                    .and_then(|o| o.as_i64())
                    .unwrap_or(0);
                let new_rot = (current_rot + degrees).rem_euclid(360);
                page_dict.set("Rotate", Object::Integer(new_rot));
                rotated_count += 1;
            }
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    doc.save(output_path).map_err(|e| format!("Failed to save rotated PDF: {e}"))?;

    Ok(PdfOperationResult {
        ok: true,
        never_files: true,
        operation: "rotate".to_string(),
        output_path: output_path.to_string_lossy().into_owned(),
        page_count: total_pages,
        message: format!("Rotated {} pages by {} degrees.", rotated_count, degrees),
    })
}

/// True byte-level text redaction: scans content streams and scrubs matching patterns
/// (SSN, phone numbers, DOBs, or user keywords) from the PDF stream bytes.
pub fn redact_pdf_text(
    input_path: &Path,
    custom_patterns: &[String],
    output_path: &Path,
) -> Result<RedactOperationResult, String> {
    if !input_path.is_file() {
        return Err(format!("Input file not found: {}", input_path.display()));
    }

    let mut doc = Document::load(input_path).map_err(|e| format!("Failed to load PDF: {e}"))?;

    // Standard court privacy regexes
    let ssn_re = regex::Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").map_err(|e| e.to_string())?;
    let phone_re = regex::Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").map_err(|e| e.to_string())?;
    let dob_re = regex::Regex::new(r"\b(0[1-9]|1[0-2])/(0[1-9]|[12]\d|3[01])/(19|20)\d{2}\b").map_err(|e| e.to_string())?;

    let mut compiled_custom = Vec::new();
    for p in custom_patterns {
        if !p.trim().is_empty() {
            if let Ok(re) = regex::Regex::new(&regex::escape(p.trim())) {
                compiled_custom.push(re);
            }
        }
    }

    let mut total_redactions = 0usize;

    // Scan all stream objects in the document
    for (_id, obj) in doc.objects.iter_mut() {
        if let Ok(stream) = obj.as_stream_mut() {
            if let Ok(content_str) = std::str::from_utf8(&stream.content) {
                let mut modified = content_str.to_string();
                let mut stream_changes = 0;

                // Scrub SSNs
                if ssn_re.is_match(&modified) {
                    for _ in ssn_re.find_iter(&modified) {
                        stream_changes += 1;
                    }
                    modified = ssn_re.replace_all(&modified, "[REDACTED-SSN]").to_string();
                }

                // Scrub phone numbers
                if phone_re.is_match(&modified) {
                    for _ in phone_re.find_iter(&modified) {
                        stream_changes += 1;
                    }
                    modified = phone_re.replace_all(&modified, "[REDACTED-PHONE]").to_string();
                }

                // Scrub DOBs
                if dob_re.is_match(&modified) {
                    for _ in dob_re.find_iter(&modified) {
                        stream_changes += 1;
                    }
                    modified = dob_re.replace_all(&modified, "[REDACTED-DOB]").to_string();
                }

                // Scrub custom words
                for re in &compiled_custom {
                    if re.is_match(&modified) {
                        for _ in re.find_iter(&modified) {
                            stream_changes += 1;
                        }
                        modified = re.replace_all(&modified, "[REDACTED]").to_string();
                    }
                }

                if stream_changes > 0 {
                    total_redactions += stream_changes;
                    stream.set_plain_content(modified.as_bytes().to_vec());
                }
            }
        }
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    doc.save(output_path).map_err(|e| format!("Failed to save redacted PDF: {e}"))?;

    Ok(RedactOperationResult {
        ok: true,
        never_files: true,
        output_path: output_path.to_string_lossy().into_owned(),
        redactions_count: total_redactions,
        message: format!("Scrubbed {} sensitive token(s) from document stream.", total_redactions),
    })
}
