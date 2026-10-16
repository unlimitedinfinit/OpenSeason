use crate::court_rules::CourtRules;
use super::types::{typst_escape, BodyBlock, CaptionStyle, CaptionVenue, CourtDocument};

pub fn to_typst(doc: &CourtDocument, rules: &CourtRules) -> String {
    let fonts = rules.typst_font_list();
    let size = rules.font_size_pt;
    // Typst `leading` is the gap between line boxes. Map Word multipliers onto a
    // consistent gap so wrapped lines match other lines in the same paragraph.
    let leading = if rules.line_spacing >= 1.8 {
        0.85
    } else if rules.line_spacing >= 1.4 {
        0.55
    } else {
        0.35
    };
    let caption = match doc.style {
        CaptionStyle::Boxed => boxed_caption_typst(doc),
        CaptionStyle::Classic => classic_caption_typst(doc),
    };
    let mut body = String::new();
    for block in &doc.body {
        match block {
            BodyBlock::Heading(h) => {
                body.push_str(&format!(
                    "\n#block(above: 1.1em, below: 0.85em)[#text(weight: \"bold\")[{}]]\n",
                    typst_escape(&h.to_ascii_uppercase())
                ));
            }
            BodyBlock::Paragraph(p) => {
                body.push_str(&format!("{}\n\n", typst_escape(p)));
            }
            BodyBlock::Blank => body.push('\n'),
        }
    }
    let mut closing = String::new();
    closing.push_str("\n#block(breakable: false)[\n#set par(leading: 0.35em, justify: false, first-line-indent: 0em)\n#v(0.7em)\n");
    if let (Some(h), Some(t)) = (&doc.certification_heading, &doc.certification_text) {
        closing.push_str(&format!(
            "#align(center)[#text(weight: \"bold\")[{}]]\n#v(0.4em)\n#set par(leading: 0.55em, justify: false)\n{}\n#v(0.75em)\n#set par(leading: 0.35em, justify: false)\n",
            typst_escape(h),
            typst_escape(t)
        ));
    }
    for line in &doc.signature_lines {
        closing.push_str(&format!("{} \\\n", typst_escape(line)));
    }
    closing.push_str("]\n");
    closing.push_str(&format!(
        "\n#pagebreak()\n#align(center)[#text(weight: \"bold\")[{}]]\n#v(0.5em)\n",
        typst_escape(&doc.certificate_heading)
    ));
    for line in &doc.certificate_lines {
        closing.push_str(&format!("{}\n\n", typst_escape(line)));
    }

    format!(
        r##"
#set page(
  paper: "{paper}",
  margin: (top: 0.75in, right: 0.85in, bottom: 0.85in, left: 0.85in),
  footer: context {{
    align(center)[#text(size: 10pt)[Page #counter(page).display() of #counter(page).final().first()]]
  }}
)
#set text(font: ({fonts}), size: {size}pt)
#set par(leading: {leading}em, spacing: {leading}em, justify: false, first-line-indent: 0em)

{caption}

#v(0.55em)
#block(stroke: 0.8pt, width: 100%, inset: 8pt)[
  #align(center)[#text(weight: "bold", size: {title_size}pt)[{title}]]
]
#v(0.7em)

{body}
{closing}
"##,
        paper = rules.paper,
        fonts = fonts,
        size = size,
        leading = leading,
        caption = caption,
        title_size = size,
        title = typst_escape(&doc.title),
        body = body,
        closing = closing,
    )
}

fn blank_or(value: &str) -> String {
    if value.trim().is_empty() {
        "_______________".into()
    } else {
        value.to_string()
    }
}

fn boxed_right_typst(doc: &CourtDocument) -> String {
    let case_no = typst_escape(&blank_or(&doc.appellate_no));
    match doc.venue {
        CaptionVenue::Colorado => {
            let lower = typst_escape(&blank_or(&doc.lower_court_name));
            let trial = typst_escape(&blank_or(&doc.trial_no));
            format!(
                r##"#align(center)[#text(weight: "bold", size: 8pt)[COURT USE ONLY]]
    #v(0.25em)
    #box(stroke: 0.5pt, width: 100%, inset: 6pt)[
      *Case Number:* {case_no} \
      Lower Court: {lower} \
      Lower Court Case: {trial}
    ]"##,
                case_no = case_no,
                lower = lower,
                trial = trial,
            )
        }
        CaptionVenue::NewJersey | CaptionVenue::Generic => {
            let trial = typst_escape(&blank_or(&doc.trial_no));
            let division_line = if doc.division_courtroom.trim().is_empty() {
                String::new()
            } else {
                format!(
                    " \\\n      Division / Courtroom: {}",
                    typst_escape(&doc.division_courtroom)
                )
            };
            format!(
                r##"#align(center)[#text(weight: "bold", size: 8pt)[COURT USE ONLY]]
    #v(0.25em)
    #box(stroke: 0.5pt, width: 100%, inset: 6pt)[
      *Case Number:* {case_no} \
      Trial Court: {trial}{division_line}
    ]"##,
                case_no = case_no,
                trial = trial,
                division_line = division_line,
            )
        }
    }
}

pub fn boxed_caption_typst(doc: &CourtDocument) -> String {
    let mut left = String::new();
    left.push_str(&format!(
        "#text(weight: \"bold\", size: 11pt)[{}]\\ \n",
        typst_escape(&doc.court_name)
    ));
    if !doc.court_division.is_empty() {
        left.push_str(&format!("{}\\ \n", typst_escape(&doc.court_division)));
    }
    for a in &doc.court_address {
        if !a.trim().is_empty() {
            left.push_str(&format!("#text(size: 9pt)[{}]\\ \n", typst_escape(a)));
        }
    }
    left.push_str("#v(0.35em)\n");
    left.push_str(&format!(
        "{}: {}\\ \nv.\\ \n{}: {}\\ \n",
        typst_escape(&doc.left_label),
        typst_escape(&doc.left_names),
        typst_escape(&doc.right_label),
        typst_escape(&doc.right_names)
    ));
    left.push_str("#v(0.3em)\n#set text(size: 9pt)\n");
    left.push_str(&format!(
        "{}, {}\\ \n",
        typst_escape(&doc.filer_name),
        typst_escape(&doc.filer_role)
    ));
    for a in &doc.filer_address {
        if !a.trim().is_empty() {
            left.push_str(&format!("{}\\ \n", typst_escape(a)));
        }
    }
    let mut contact = Vec::new();
    if !doc.filer_email.is_empty() {
        contact.push(doc.filer_email.clone());
    }
    if !doc.filer_phone.is_empty() {
        contact.push(doc.filer_phone.clone());
    }
    if !contact.is_empty() {
        left.push_str(&format!("{}\\ \n", typst_escape(&contact.join(" · "))));
    }

    let right = boxed_right_typst(doc);

    format!(
        r##"
#table(
  columns: (1fr, 2.35in),
  stroke: 0.8pt,
  inset: 6pt,
  [
    #set par(leading: 0.38em, justify: false)
    #set text(size: 9.5pt)
    {left}
  ],
  [
    #set par(leading: 0.38em, justify: false)
    #set text(size: 9pt)
    {right}
  ]
)
"##,
        left = left,
        right = right,
    )
}

pub fn classic_caption_typst(doc: &CourtDocument) -> String {
    let mut s = String::from("#align(center)[\n#set par(leading: 0.55em, justify: false)\n");
    s.push_str(&format!(
        "#text(weight: \"bold\")[{}]\\ \n",
        typst_escape(&doc.court_name)
    ));
    if !doc.court_division.is_empty() {
        s.push_str(&format!("{}\\ \n", typst_escape(&doc.court_division)));
    }
    s.push_str("#v(0.4em)\n");
    s.push_str(&format!("{}\\ \n", typst_escape(&doc.left_names)));
    s.push_str(&format!("{}(s),\\ \n", typst_escape(&doc.left_label)));
    s.push_str("v.\\ \n");
    s.push_str(&format!("{}\\ \n", typst_escape(&doc.right_names)));
    s.push_str(&format!("{}(s).\\ \n", typst_escape(&doc.right_label)));
    if !doc.appellate_no.is_empty() {
        s.push_str(&format!("Case No. {}\\ \n", typst_escape(&doc.appellate_no)));
    }
    s.push_str("]\n");
    s
}
