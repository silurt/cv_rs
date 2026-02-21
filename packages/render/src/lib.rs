use core::schema::types::CVSchema;

use oxidize_pdf::{Document, Font, Page, Result, TextAlign, TextFlowContext};

pub fn render(schema: &CVSchema) -> Result<Document> {
    let mut doc = Document::new();
    doc.set_title(&format!("{}'s CV", &schema.person.name));

    let mut page = Page::a4();
    let mut flow = page.text_flow();
    flow.at(50.0, 750.0)
        .set_font(Font::Helvetica, 10.0)
        .set_alignment(TextAlign::Left);
    // .write_wrapped(&cv_text(schema))?;

    for element in cv_text(schema) {
        match element {
            Element::Body(text) => {
                flow.write_wrapped(&text)?;
            }
            Element::Newline => {
                flow.write_wrapped("\n")?;
            } // Element::Image(data) => {
              //     // For future use
              // }
        }
        // println!("Current flow position: {:?}", flow.cursor_position());
        // Check if the current flow position is near the bottom of the page, and if so, add a new page
        if flow.cursor_position().1 < 50.0 {
            // doc.add_page(page);

            println!("Adding new page...");

            page.add_text_flow(&flow);
            doc.add_page(page);

            page = Page::a4();
            flow = page.text_flow();

            flow.at(50.0, 750.0)
                .set_font(Font::Helvetica, 10.0)
                .set_alignment(TextAlign::Left);
        }
    }

    page.add_text_flow(&flow);
    doc.add_page(page);

    Ok(doc)
}

// fn push_text(flow: &mut TextFlowContext, text: &str) -> Result<()> {
//     flow.write_wrapped(text)?;

//     Ok(())
// }

// struct Element {
//     text: String,
//     font: Font,
//     size: f32,
//     align: TextAlign,
// }

enum Element {
    Body(String),
    Newline,
    // Image(Vec<u8>), // For future use
}

fn cv_text(s: &CVSchema) -> Vec<Element> {
    let mut out: Vec<Element> = vec![];

    // Person
    out.push(Element::Body(format!(
        "{} | {} | {} | {}\n\n",
        s.person.name, s.person.location, s.person.email, s.person.phone
    )));

    // Links
    out.push(Element::Body(format!(
        "GitHub: {}  LinkedIn: {}  Portfolio: {}\n\n",
        s.links.github, s.links.linkedin, s.links.portfolio
    )));

    // Profile
    out.push(Element::Body(format!("PROFILE\n{}\n\n", s.profile)));
    // Core competencies
    out.push(Element::Body(format!(
        "CORE COMPETENCIES\n{}\n\n",
        s.core_competencies.join(", ")
    )));

    // Technical focus areas
    out.push(Element::Body(format!(
        "TECHNICAL FOCUS AREAS\n{}\n\n",
        s.technical_focus_areas.join(", ")
    )));

    // Key achievements
    out.push(Element::Body("KEY ACHIEVEMENTS\n".to_string()));
    for a in &s.key_achievements {
        out.push(Element::Body(format!("- {}\n", a)));
    }
    out.push(Element::Newline);

    // Tech leadership
    out.push(Element::Body("TECH LEADERSHIP\n".to_string()));
    for l in &s.tech_leadership {
        out.push(Element::Body(format!("- {}\n", l)));
    }
    out.push(Element::Newline);

    // Selected projects
    out.push(Element::Body("SELECTED PROJECTS\n".to_string()));
    for p in &s.selected_projects {
        out.push(Element::Body(format!(
            "{} ({}): {}\n",
            p.name, p.date_range, p.description
        )));
    }
    out.push(Element::Newline);
    // Early career
    out.push(Element::Body(format!(
        "EARLY CAREER ({})\n{}\n\n",
        s.early_career.date_range, s.early_career.summary
    )));

    // Experience
    out.push(Element::Body("EXPERIENCE\n".to_string()));
    for e in &s.experience {
        out.push(Element::Body(format!(
            "{} - {} | {} | {} to {}\n{}\n",
            e.company, e.role, e.location, e.start_date, e.end_date, e.summary
        )));
        for h in &e.highlights {
            out.push(Element::Body(format!("  - {}\n", h)));
        }
        out.push(Element::Newline);
    }

    // Technical skills
    let ts = &s.technical_skills;
    out.push(Element::Body("TECHNICAL SKILLS\n".to_string()));
    out.push(Element::Body(format!(
        "Languages: {}\n",
        ts.languages.join(", ")
    )));
    out.push(Element::Body(format!(
        "Frameworks: {}\n",
        ts.frameworks.join(", ")
    )));
    out.push(Element::Body(format!(
        "AI/ML: {}\n",
        ts.ai_ml_skills.join(", ")
    )));
    out.push(Element::Body(format!(
        "Backend/Cloud: {}\n",
        ts.backend_cloud_skills.join(", ")
    )));
    out.push(Element::Body(format!(
        "Blockchain: {}\n",
        ts.blockchain_skills.join(", ")
    )));
    out.push(Element::Body(format!(
        "Mobile: {}\n",
        ts.mobile_skills.join(", ")
    )));
    out.push(Element::Body(format!("Tools: {}\n\n", ts.tools.join(", "))));

    // Languages
    out.push(Element::Body("LANGUAGES\n".to_string()));
    for l in &s.languages {
        out.push(Element::Body(format!("{}: {}\n", l.name, l.proficiency)));
    }
    out.push(Element::Newline);

    // Education
    out.push(Element::Body("EDUCATION\n".to_string()));
    for e in &s.education {
        out.push(Element::Body(format!(
            "{} - {} in {} | {} | {} to {}\n",
            e.institution, e.degree, e.field, e.location, e.start_date, e.end_date
        )));
    }

    out
}
