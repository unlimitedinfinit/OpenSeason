//! Machine-readable capabilities manifest for agents and CLI tool discovery.
//!
//! Exposes tool contracts, offline safety flags, approval requirements,
//! and parameters so external agents can inspect what JustLegal OS can do.

use serde::{Deserialize, Serialize};
use crate::check::NEVER_FILE_NOTICE;
use crate::forms::LEGAL_INFO_NOTICE;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityItem {
    pub command: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub offline_safe: bool,
    pub modifies_filesystem: bool,
    pub requires_user_approval: bool,
    pub requires_active_case: bool,
    pub parameters: Vec<CapabilityParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilitiesManifest {
    pub product: String,
    pub version: String,
    pub engine: String,
    pub notice: String,
    pub total_tools: usize,
    pub capabilities: Vec<CapabilityItem>,
}

pub fn list_capabilities() -> CapabilitiesManifest {
    let capabilities = vec![
        // -------------------------------------------------------------------
        // Category 1: Case Workspace
        // -------------------------------------------------------------------
        CapabilityItem {
            command: "case context".into(),
            name: "Get Consolidated Case Context".into(),
            description: "1-call consolidated case bundle containing profile, venue competence, deadlines, pending intake, and exhibits index.".into(),
            category: "Case Workspace".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Path to active case directory containing case.json.".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "case create".into(),
            name: "Create Case Matter".into(),
            description: "Initialize a new standard or confidential legal workspace folder with case.json profile.".into(),
            category: "Case Workspace".into(),
            offline_safe: true,
            modifies_filesystem: true,
            requires_user_approval: true,
            requires_active_case: false,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Path where case directory will be initialized.".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "title".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "Case title (e.g. Plaintiff v. Defendant or Matter Description).".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "mode".into(),
                    param_type: "string".into(),
                    required: false,
                    description: "Operating mode: 'standard' or 'confidential'.".into(),
                    default_value: Some("standard".into()),
                },
                CapabilityParameter {
                    name: "kind".into(),
                    param_type: "string".into(),
                    required: false,
                    description: "Matter kind (e.g. 'motion', 'appellate_appeal', 'will', 'lease').".into(),
                    default_value: Some("motion".into()),
                },
            ],
        },
        CapabilityItem {
            command: "case next-steps".into(),
            name: "Predict Next Procedural Steps".into(),
            description: "Examine case timeline, orders, and jurisdiction to predict upcoming procedural branches and statutory deadlines.".into(),
            category: "Case Workspace".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Path to case directory.".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "case seal".into(),
            name: "Seal Case Locally".into(),
            description: "Mark case folder as confidential (.sealed) and refuse all future network sync, backup, or publish requests.".into(),
            category: "Case Workspace".into(),
            offline_safe: true,
            modifies_filesystem: true,
            requires_user_approval: true,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Path to case directory to seal.".into(),
                    default_value: None,
                },
            ],
        },

        // -------------------------------------------------------------------
        // Category 2: Forms & Templates Library
        // -------------------------------------------------------------------
        CapabilityItem {
            command: "forms search".into(),
            name: "Search Forms & Templates".into(),
            description: "Search across official court forms, everyday legal agreements, and templates by task or keyword.".into(),
            category: "Forms Library".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: false,
            parameters: vec![
                CapabilityParameter {
                    name: "query".into(),
                    param_type: "string".into(),
                    required: false,
                    description: "Keyword to search (e.g. 'lease', 'custody', 'fee waiver', 'reconsideration').".into(),
                    default_value: Some("".into()),
                },
                CapabilityParameter {
                    name: "state".into(),
                    param_type: "string".into(),
                    required: false,
                    description: "State 2-letter abbreviation (e.g. 'NJ', 'CO').".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "task".into(),
                    param_type: "string".into(),
                    required: false,
                    description: "Task category (e.g. 'housing', 'family', 'wills', 'contracts', 'court').".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "forms recommend".into(),
            name: "Recommend Forms for Active Case".into(),
            description: "Case-aware discovery: inspects case state, court, and procedural stage to prioritize relevant forms.".into(),
            category: "Forms Library".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "case".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Path to active case workspace.".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "forms inspect".into(),
            name: "Inspect Form Metadata & Fields".into(),
            description: "Inspect form teaching (what it is, when to use, warnings), official court URL, SHA-256 hash, and fill mapping.".into(),
            category: "Forms Library".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: false,
            parameters: vec![
                CapabilityParameter {
                    name: "id".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "Unique form identifier (e.g. 'nj-cn-10837', 'jdf-205').".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "forms fetch".into(),
            name: "Download & Verify Form".into(),
            description: "Fetch a verified official form PDF or template into the offline Library with strict SHA-256 integrity verification.".into(),
            category: "Forms Library".into(),
            offline_safe: false,
            modifies_filesystem: true,
            requires_user_approval: true,
            requires_active_case: false,
            parameters: vec![
                CapabilityParameter {
                    name: "id".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "Form ID to fetch.".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "network".into(),
                    param_type: "boolean".into(),
                    required: false,
                    description: "Must be true to allow network download from verified mirrors.".into(),
                    default_value: Some("false".into()),
                },
            ],
        },
        CapabilityItem {
            command: "forms attach".into(),
            name: "Attach Form to Case".into(),
            description: "Attach form to active case workspace: creates editable draft in drafts/ with prefilled caption, or copies verified exhibit to evidence/.".into(),
            category: "Forms Library".into(),
            offline_safe: true,
            modifies_filesystem: true,
            requires_user_approval: true,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "id".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "Form ID to attach.".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "case".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Path to active case workspace.".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "as_template".into(),
                    param_type: "boolean".into(),
                    required: false,
                    description: "True to create editable draft in drafts/; false to attach as exhibit.".into(),
                    default_value: Some("true".into()),
                },
            ],
        },
        CapabilityItem {
            command: "packs list".into(),
            name: "List Form Packs".into(),
            description: "List available and locally installed court form packs with size labels and offline statuses.".into(),
            category: "Forms Library".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: false,
            parameters: vec![],
        },
        CapabilityItem {
            command: "packs install".into(),
            name: "Install Pack Archive".into(),
            description: "Install a state court forms pack locally from an archive file with SHA-256 verification.".into(),
            category: "Forms Library".into(),
            offline_safe: true,
            modifies_filesystem: true,
            requires_user_approval: true,
            requires_active_case: false,
            parameters: vec![
                CapabilityParameter {
                    name: "state".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "State abbreviation (e.g. NJ, CO).".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "file".into(),
                    param_type: "path".into(),
                    required: false,
                    description: "Local zip archive or pack directory path.".into(),
                    default_value: None,
                },
            ],
        },

        // -------------------------------------------------------------------
        // Category 3: Drafting & Formatting
        // -------------------------------------------------------------------
        CapabilityItem {
            command: "draft import".into(),
            name: "Import Markdown Draft".into(),
            description: "Import a markdown draft into case, separating caption front-matter from document body.".into(),
            category: "Drafting & Formatting".into(),
            offline_safe: true,
            modifies_filesystem: true,
            requires_user_approval: true,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Destination case workspace.".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "md".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Path to Markdown file to import.".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "draft preview".into(),
            name: "Preview Formatted Draft".into(),
            description: "Preview court caption, certificate of service, and body layout without writing files.".into(),
            category: "Drafting & Formatting".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: false,
            parameters: vec![
                CapabilityParameter {
                    name: "md".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Path to Markdown draft file.".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "export court-md".into(),
            name: "Export Court-Formatted Word & PDF".into(),
            description: "Assemble and export court-formatted Word (.docx) and Typst PDF from imported draft and case profile.".into(),
            category: "Drafting & Formatting".into(),
            offline_safe: true,
            modifies_filesystem: true,
            requires_user_approval: true,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Case workspace folder.".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "service cert".into(),
            name: "Generate Certificate of Service".into(),
            description: "Generate court-formatted Certificate of Service PDF with filer verification.".into(),
            category: "Drafting & Formatting".into(),
            offline_safe: true,
            modifies_filesystem: true,
            requires_user_approval: true,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Case workspace folder.".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "sign".into(),
            name: "Stamp Electronic Signature".into(),
            description: "Stamp typed /s/ electronic signature block onto court filing PDF.".into(),
            category: "Drafting & Formatting".into(),
            offline_safe: true,
            modifies_filesystem: true,
            requires_user_approval: true,
            requires_active_case: false,
            parameters: vec![
                CapabilityParameter {
                    name: "pdf".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Target PDF file to sign.".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "name".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "Signer name.".into(),
                    default_value: None,
                },
            ],
        },

        // -------------------------------------------------------------------
        // Category 4: Verification & Integrity
        // -------------------------------------------------------------------
        CapabilityItem {
            command: "check venue".into(),
            name: "Verify Venue & Competence Rules".into(),
            description: "Check venue-specific and tier-specific competence rules (e.g. NJ R. 1:4-4b, CO Rule 121 conferral, 28 USC 1746).".into(),
            category: "Verification & Integrity".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Case workspace folder.".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "check deadline".into(),
            name: "Calculate Rule-Based Deadline".into(),
            description: "Honest deadline calculation from trigger/order date using official court rule and calendar math. Never invents dates.".into(),
            category: "Verification & Integrity".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Case workspace folder.".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "rule".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "Rule identifier from rule pack.".into(),
                    default_value: None,
                },
                CapabilityParameter {
                    name: "trigger_date".into(),
                    param_type: "string".into(),
                    required: true,
                    description: "Order or service trigger date (YYYY-MM-DD).".into(),
                    default_value: None,
                },
            ],
        },
        CapabilityItem {
            command: "check all".into(),
            name: "Run Complete Preflight Audit".into(),
            description: "Comprehensive check of all fact cites, legal authorities, court deadlines, and venue rules before export.".into(),
            category: "Verification & Integrity".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: true,
            parameters: vec![
                CapabilityParameter {
                    name: "path".into(),
                    param_type: "path".into(),
                    required: true,
                    description: "Case workspace folder.".into(),
                    default_value: None,
                },
            ],
        },

        // -------------------------------------------------------------------
        // Category 5: Multi-Matter Portfolio
        // -------------------------------------------------------------------
        CapabilityItem {
            command: "portfolio status".into(),
            name: "Portfolio Radar & Filer Profile".into(),
            description: "Radar view across all local cases, upcoming statutory deadlines, and global filer contact profile.".into(),
            category: "Portfolio & Multi-Matter".into(),
            offline_safe: true,
            modifies_filesystem: false,
            requires_user_approval: false,
            requires_active_case: false,
            parameters: vec![],
        },
    ];

    let count = capabilities.len();

    CapabilitiesManifest {
        product: "JustLegal Desktop".into(),
        version: "0.1.0".into(),
        engine: "JustLegal OS / OpenSeason Engine".into(),
        notice: format!("{NEVER_FILE_NOTICE}\n{LEGAL_INFO_NOTICE}"),
        total_tools: count,
        capabilities,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_manifest_lists_all_tools_honestly() {
        let manifest = list_capabilities();
        assert_eq!(manifest.product, "JustLegal Desktop");
        assert!(manifest.total_tools >= 15);
        assert!(manifest.capabilities.iter().any(|c| c.command == "forms search"));
        assert!(manifest.capabilities.iter().any(|c| c.command == "forms recommend"));
        assert!(manifest.capabilities.iter().any(|c| c.command == "forms fetch"));
        assert!(manifest.capabilities.iter().any(|c| c.command == "forms attach"));
        assert!(manifest.capabilities.iter().any(|c| c.command == "case context"));
        assert!(manifest.capabilities.iter().any(|c| c.command == "check venue"));
    }
}
