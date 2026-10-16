pub mod types;
pub mod assembly;
pub mod writers;
pub mod exporter;

#[cfg(test)]
mod tests;

pub use types::{pleading_heading, typst_escape, ExportPaths};

pub use assembly::{assembled_notice_paragraphs, assembled_pleading_paragraphs};

pub use writers::{inspect_docx_contains, inspect_pdf_contains};

pub use exporter::{
    export_court_markdown, export_custom_filing, export_notice_of_appeal,
    export_notice_of_appeal_docx_only, export_pleading, write_lines_as_docx_pdf,
};
