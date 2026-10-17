use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "openseason",
    about = "Offline Case Builder / Document Builder. Formatting and checking only. Never files or serves.",
    long_about = "Create a portable case folder, import a Markdown draft, run record/authority/deadline checks, and export Word and PDF.\n\nHard rule: this tool only formats and checks the user's own material. Read AGENTS.md and docs/AGENT-INTERFACE.md before using it from an AI agent."
)]
pub struct Cli {
    /// Print machine-readable JSON (agent interface)
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Case {
        #[command(subcommand)]
        action: CaseCmd,
    },
    Draft {
        #[command(subcommand)]
        action: DraftCmd,
    },
    Evidence {
        #[command(subcommand)]
        action: EvidenceCmd,
    },
    Check {
        #[command(subcommand)]
        action: CheckCmd,
    },
    Export {
        #[command(subcommand)]
        action: ExportCmd,
    },
    Packs {
        #[command(subcommand)]
        action: PacksCmd,
    },
    /// What do I need to file? Plain-language finder. Legal information, not advice.
    Guide {
        #[command(subcommand)]
        action: GuideCmd,
    },
    Gaps {
        #[command(subcommand)]
        action: GapsCmd,
    },
    /// Print the agent playbook, field help, or JustLegal resource links (data only)
    Docs {
        #[command(subcommand)]
        action: DocsCmd,
    },
    /// MCP stdio server (same engine as the CLI)
    Mcp,
    /// Agent contract: tool list, MCP launch, starter prompt
    Agent {
        #[command(subcommand)]
        action: AgentCmd,
    },
    /// Agent capabilities manifest (JSON)
    Capabilities,
    Sync {
        path: PathBuf,
        #[arg(long, default_value = "backup")]
        action: String,
    },
    /// JustLegal Cloud sync and authentication
    Cloud {
        #[command(subcommand)]
        action: CloudCmd,
    },
    /// Official court form AcroForm inspection and auto-filling
    Form {
        #[command(subcommand)]
        action: FormCmd,
    },
    /// Manage multiple motions and filings in the case docket
    Motion {
        #[command(subcommand)]
        action: MotionCmd,
    },
    /// Generate court Certificate of Service and Proposed Form of Order
    Service {
        #[command(subcommand)]
        action: ServiceCmd,
    },
    /// Offline local PDF power tools (merge, split, rotate, redact)
    Pdf {
        #[command(subcommand)]
        action: PdfCmd,
    },
    /// Stamp court-compliant electronic signature block onto a PDF
    Sign {
        pdf: PathBuf,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "0")]
        page: u32,
        #[arg(long, default_value = "72")]
        x: f32,
        #[arg(long, default_value = "100")]
        y: f32,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Guided 1-question-at-a-time Intake Wizard & Offline PDF Ingestion Scanner
    Intake {
        #[command(subcommand)]
        action: IntakeCmd,
    },
    /// Multi-state Portfolio Deadline Radar & Global Filer Profile
    Portfolio {
        #[command(subcommand)]
        action: PortfolioCmd,
    },
    /// Reconstruct, verify, and reconcile the official court record
    Record {
        #[command(subcommand)]
        action: RecordCmd,
    },
    /// Review orders against record to isolate discrepancies
    Review {
        #[command(subcommand)]
        action: ReviewCmd,
    },
    /// Accountability routes, oversight forms, and ethics guidance
    Accountability {
        #[command(subcommand)]
        action: AccountabilityCmd,
    },
    /// Title II ADA Equal Access & Court Accommodation Request
    Ada {
        #[command(subcommand)]
        action: AdaCmd,
    },
    /// Decode court orders and notices (detect signer, operative clauses, candidate deadlines)
    Order {
        #[command(subcommand)]
        action: OrderCmd,
    },
    /// Audit mailing postmarks vs clerk receipt timestamps for jurisdictional traps
    Timeline {
        #[command(subcommand)]
        action: TimelineCmd,
    },
    /// Court access directory (clerk counter hours, e-filing portals, docket search, fee schedules)
    Court {
        #[command(subcommand)]
        action: CourtCmd,
    },
    /// Agent Workflow Coordinator: Formulate plans, advance steps, and resume uninterrupted
    Workflow {
        #[command(subcommand)]
        action: WorkflowCmd,
    },
    /// Qui Tam & False Claims Act (31 U.S.C. § 3729) fraud triage under EO 14395
    QuiTam {
        #[command(subcommand)]
        action: QuiTamCmd,
    },
    /// System health, local workspace, packs, and airgap verification diagnostic
    Doctor,
}


#[derive(Subcommand)]
pub enum IntakeCmd {
    Status {
        path: PathBuf,
    },
    Answer {
        path: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        answer: String,
    },
    Scan {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum PortfolioCmd {
    Status {
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum CaseCmd {
    Create {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "standard")]
        mode: String,
        #[arg(long, default_value = "motion")]
        kind: String,
        #[arg(long, default_value = "generic-us-appellate")]
        pack: String,
        #[arg(long, default_value = "")]
        motion_kind: String,
    },
    Validate {
        path: PathBuf,
    },
    Seal {
        path: PathBuf,
    },
    /// Copy the fake Lucille Example reconsideration case
    Seed {
        #[arg(long)]
        path: PathBuf,
        #[arg(long, default_value = "lucille")]
        template: String,
    },
    /// Copy a folder of drafts/notes/orders into a case. Never moves originals. Does not auto-commit guesses.
    ImportFolder {
        /// Destination case folder (created if missing)
        path: PathBuf,
        /// Source folder of drafts, notes, and orders
        #[arg(long = "from")]
        from: PathBuf,
        #[arg(long, default_value = "")]
        title: String,
    },
    /// Consolidated case context bundle (profile + venue + next-steps + intake + exhibits)
    Context {
        path: PathBuf,
    },
    /// Predict downstream procedural branches and deadlines from timeline & orders
    NextSteps {
        path: PathBuf,
    },
    /// Graduate Tier 2 -> Tier 3 Lawsuit or Escalate Trial -> Appeal / Mandamus / SCOTUS
    Escalate {
        path: PathBuf,
        #[arg(long, default_value = "appellate")]
        to: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Bootstrap and rescue a case from an unorganized folder of documents
    Bootstrap {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        to: PathBuf,
        #[arg(long)]
        title: Option<String>,
    },
}


#[derive(Subcommand)]
pub enum DraftCmd {
    Import {
        path: PathBuf,
        #[arg(long)]
        md: PathBuf,
    },
    /// Preview a Markdown draft as a court caption + body. Does not write files.
    Preview {
        path: Option<PathBuf>,
        #[arg(long)]
        md: Option<PathBuf>,
    },
    Sections {
        path: PathBuf,
    },
    Fill {
        path: PathBuf,
        #[arg(long)]
        id: String,
        #[arg(long)]
        text: String,
    },
}

#[derive(Subcommand)]
pub enum EvidenceCmd {
    Add {
        path: PathBuf,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value = "")]
        description: String,
    },
    Index {
        path: PathBuf,
    },
    Link {
        path: PathBuf,
        #[arg(long)]
        fact_id: u32,
        #[arg(long)]
        letter: String,
    },
}

#[derive(Subcommand)]
pub enum CheckCmd {
    Fact {
        path: PathBuf,
        #[arg(long)]
        id: Option<u32>,
        #[arg(long)]
        cite: Option<String>,
        #[arg(long)]
        exhibit: Option<String>,
        #[arg(long)]
        verified: bool,
    },
    Authority {
        path: PathBuf,
        #[arg(long)]
        id: Option<u32>,
        #[arg(long, default_value = "unverified")]
        status: String,
        #[arg(long, default_value = "")]
        source: String,
        #[arg(long, default_value = "")]
        note: String,
    },
    Deadline {
        path: PathBuf,
        #[arg(long)]
        pack: String,
        #[arg(long)]
        rule: String,
        #[arg(long)]
        trigger_date: String,
    },
    Signoff {
        path: PathBuf,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        checked: bool,
    },
    All {
        path: PathBuf,
        #[arg(long)]
        pack: Option<String>,
        #[arg(long)]
        rule: Option<String>,
        #[arg(long)]
        trigger_date: Option<String>,
    },
    Preflight {
        path: PathBuf,
    },
    Transcripts {
        path: PathBuf,
    },
    /// Check venue-specific and tier-specific competence rules (NJ R. 1:4-4b, CO Rule 121, 28 USC 1746, SCOTUS, Wills/Leases)
    Venue {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ExportCmd {
    NoticeOfAppeal {
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Pleading {
        path: PathBuf,
        #[arg(long, default_value = "motion")]
        kind: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    CourtMd {
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Packet {
        path: PathBuf,
    },
    ExhibitPacket {
        path: PathBuf,
        #[arg(long)]
        prefix: Option<String>,
        #[arg(long)]
        start: Option<u32>,
    },
    Ics {
        path: PathBuf,
        #[arg(long)]
        pack: String,
        #[arg(long)]
        rule: String,
        #[arg(long)]
        trigger_date: String,
    },
    /// Generate 2-page executive Attorney Consultation Dossier with verified citations and stress-tests
    ConsultationDossier {
        path: PathBuf,
        #[arg(long = "question", action = clap::ArgAction::Append)]
        questions: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum CloudCmd {
    Login {
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    Status,
    Logout,
    Pull {
        #[arg(long)]
        case: Option<String>,
        #[arg(long)]
        root: Option<PathBuf>,
    },
    Push {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum FormCmd {
    Inspect {
        pdf: PathBuf,
        #[arg(long)]
        case: Option<PathBuf>,
    },
    Fill {
        pdf: PathBuf,
        #[arg(long)]
        case: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum MotionCmd {
    List {
        path: PathBuf,
    },
    Create {
        path: PathBuf,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "motion")]
        kind: String,
        #[arg(long, default_value = "")]
        rule: String,
        #[arg(long, default_value = "")]
        text: String,
    },
    Export {
        path: PathBuf,
        #[arg(long)]
        id: String,
    },
    /// Evaluate extension options: 30-day consent stipulation letter vs formal pro se motion CN 10503
    Extension {
        path: PathBuf,
        #[arg(long, default_value = "")]
        target: String,
        #[arg(long, default_value = "30")]
        days: u32,
        #[arg(long)]
        consent: bool,
        #[arg(long)]
        suggest: bool,
    },
    /// Audit case baseline readiness and suggest 30-day extension with conferral and formal motion drafts
    ExtensionSuggest {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ServiceCmd {
    Cert {
        path: PathBuf,
        #[arg(long, default_value = "the accompanying papers", alias = "title")]
        filing: String,
        #[arg(long, default_value = "eCourts / Electronic Filing System")]
        method: String,
        #[arg(long, default_value = "")]
        date: String,
        #[arg(long, default_value = "")]
        signer: String,
        #[arg(long = "recipient", action = clap::ArgAction::Append)]
        recipients: Vec<String>,
    },
    Order {
        path: PathBuf,
        #[arg(long, default_value = "RELIEF REQUESTED", alias = "motion")]
        title: String,
        #[arg(long, default_value = "")]
        relief: String,
    },
}

#[derive(Subcommand)]
pub enum PdfCmd {
    Merge {
        #[arg(long)]
        out: PathBuf,
        files: Vec<PathBuf>,
    },
    Split {
        #[arg(long)]
        pdf: PathBuf,
        #[arg(long)]
        pages: String,
        #[arg(long)]
        out: PathBuf,
    },
    Rotate {
        #[arg(long)]
        pdf: PathBuf,
        #[arg(long, default_value = "90")]
        degrees: i64,
        #[arg(long)]
        out: PathBuf,
    },
    Redact {
        #[arg(long)]
        pdf: PathBuf,
        #[arg(long)]
        words: Vec<String>,
        #[arg(long)]
        out: PathBuf,
    },
    /// Extract text offline from a PDF file using lopdf
    ExtractText {
        pdf: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum PacksCmd {
    List {
        #[arg(long)]
        catalog: Option<PathBuf>,
    },
    Install {
        state: String,
        #[arg(long)]
        situation: Option<String>,
        #[arg(long)]
        catalog: Option<PathBuf>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    Remove {
        state: String,
    },
    Verify {
        state: Option<String>,
    },
    Validate {
        path: PathBuf,
    },
    Catalog {
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum GuideCmd {
    Questions,
    Find {
        #[arg(long)]
        state: String,
        #[arg(long)]
        court: Option<String>,
        #[arg(long)]
        situation: String,
        #[arg(long)]
        trigger_date: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum GapsCmd {
    Add {
        path: PathBuf,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        court: String,
        #[arg(long, default_value = "")]
        rule: String,
        #[arg(long, default_value = "")]
        happened: String,
        #[arg(long, default_value = "")]
        hard: String,
    },
    Export {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum DocsCmd {
    Playbook,
    Fields,
    /// JustLegal help/resource URLs as data. Never opens a browser.
    Links,
}

#[derive(Subcommand)]
pub enum AgentCmd {
    /// List every CLI/MCP tool, MCP launch config, and a copyable starter prompt
    Manifest,
    /// List complete machine-readable capabilities manifest
    Capabilities,
}

#[derive(Subcommand)]
pub enum RecordCmd {
    /// Scan and inventory files, classifying into 4 evidentiary tiers
    Inventory {
        path: PathBuf,
    },
    /// Reconcile docket sheet entries against local files
    Reconcile {
        path: PathBuf,
        #[arg(long)]
        docket: Option<PathBuf>,
    },
    /// List missing orders, opposition filings, or hearing transcripts
    Missing {
        path: PathBuf,
    },
    /// Generate official court records request form (e.g. NJ Form CN 10200)
    RequestForm {
        path: PathBuf,
        #[arg(long)]
        state: Option<String>,
    },
    /// Plan retrieval of missing records using court rules and directory
    PlanRetrieval {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ReviewCmd {
    /// Audit judicial orders against verified record for unsupported premises
    Record {
        path: PathBuf,
    },
    /// Stress-test case issues against devil's advocate counter-arguments and procedural hurdles
    StressTest {
        path: PathBuf,
        #[arg(long)]
        issue: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AccountabilityCmd {
    /// Map identified case issues to procedural review vs. misconduct routes
    Routes {
        path: PathBuf,
        #[arg(long)]
        issue: Option<String>,
    },
    /// Discover official oversight bodies, forms, portals, and rules
    Resources {
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        category: Option<String>,
    },
    /// Assemble factual, exhibit-grounded submission packet for an issue
    Packet {
        path: PathBuf,
        #[arg(long)]
        issue: String,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum AdaCmd {
    /// Generate Title II ADA formal administrative accommodation petition
    Petition {
        path: PathBuf,
        #[arg(long)]
        need: String,
        #[arg(long)]
        remote: bool,
        #[arg(long)]
        extended_time: bool,
        #[arg(long)]
        mobility: bool,
        #[arg(long)]
        hearing: bool,
    },
}

#[derive(Subcommand)]
pub enum OrderCmd {
    /// Decode a court order or notice from a PDF file, optionally ingesting it directly into a case folder
    Decode {
        #[arg(long)]
        pdf: PathBuf,
        #[arg(long)]
        case: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum TimelineCmd {
    /// Audit order entry date, postmark, and clerk receipt for mail delay traps
    AuditMail {
        path: PathBuf,
        #[arg(long)]
        order_date: String,
        #[arg(long)]
        postmark: Option<String>,
        #[arg(long)]
        delivery: Option<String>,
        #[arg(long)]
        clerk_date: Option<String>,
        #[arg(long)]
        state: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CourtCmd {
    /// List all court access profiles
    List,
    /// View detailed court access profile, clerk hours, and rules
    Info {
        query: String,
    },
}

#[derive(Subcommand)]
pub enum WorkflowCmd {
    /// Formulate an actionable multi-step legal workflow plan from a natural language objective
    Plan {
        path: PathBuf,
        #[arg(long)]
        objective: String,
    },
    /// Inspect active workflow plan, current step, and pending human approval
    Status {
        path: PathBuf,
    },
    /// Advance active workflow plan to the next step, optionally providing human approval
    Advance {
        path: PathBuf,
        #[arg(long)]
        approve: bool,
    },
}

#[derive(Subcommand)]
pub enum QuiTamCmd {
    /// Verify target entity across CMS NPPES Registry and USAspending federal awards
    VerifyEntity {
        #[arg(long)]
        name: String,
        #[arg(long)]
        npi: Option<String>,
        #[arg(long)]
        state: Option<String>,
    },
    /// Evaluate case against 31 U.S.C. § 3729 & FRCP Rule 9(b) 6-element viability standard
    #[command(name = "audit-9b")]
    Audit9b {
        path: PathBuf,
    },
    /// Ingest observational photos/videos/records with EXIF metadata and SHA-256 chain of custody
    IngestObservation {
        path: PathBuf,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value = "")]
        notes: String,
        #[arg(long)]
        observer: Option<String>,
    },
    /// Generate jurisdiction-specific licensing, fire inspection, or Medicaid records request (MGDPA/OPRA/CORA/CPRA/FOIA)
    RecordsPack {
        path: PathBuf,
        #[arg(long)]
        state: Option<String>,
    },
    /// Compile the DOJ 60-Day Triage & Relator Disclosure Packet (JSON manifest + Typst PDF)
    ExportDisclosure {
        path: PathBuf,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        attestation: Option<String>,
    },
    /// Format agency web portal auto-fill blocks (character-limited copy-paste blocks for HHS-OIG, DOJ, etc.)
    #[command(name = "format-portal")]
    FormatPortal {
        path: PathBuf,
        #[arg(long, default_value = "HHS")]
        agency: String,
    },
    /// Format ABA Rule 1.18 privileged Qui Tam Attorney Inquiry Package with Calling Card header
    #[command(name = "format-inquiry")]
    FormatInquiry {
        path: PathBuf,
    },
}
