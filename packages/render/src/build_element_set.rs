use core::schema::types::CVSchema;

use crate::template::types::{Element, ElementSet};

pub fn build_element_set(s: &CVSchema) -> ElementSet {
    let mut out = ElementSet::new();

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
