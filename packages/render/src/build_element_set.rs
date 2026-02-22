use core::schema::types::CVSchema;

use crate::template::types::{Element, ElementSet};

pub fn build_element_set(s: &CVSchema) -> ElementSet {
    let mut out = ElementSet::new();

    // Person
    out.push(Element::Header(s.person.clone(), s.links.clone()));

    // Profile
    out.push(Element::Title("PROFILE".to_owned()));
    out.push(Element::Body(s.profile.clone()));

    // Core competencies
    out.push(Element::Title("CORE COMPETENCIES".to_string()));
    out.push(Element::Tags(s.core_competencies.clone()));

    // Technical focus areas
    out.push(Element::Title("TECHNICAL FOCUS AREAS".to_string()));
    out.push(Element::Tags(s.technical_focus_areas.clone()));

    // Key achievements
    out.push(Element::Title("KEY ACHIEVEMENTS".to_string()));
    out.push(Element::List(s.key_achievements.clone()));

    // Tech leadership
    out.push(Element::Title("TECH LEADERSHIP".to_string()));
    out.push(Element::List(s.tech_leadership.clone()));

    // Selected projects
    out.push(Element::Title("SELECTED PROJECTS".to_string()));
    out.push(Element::List(
        s.selected_projects
            .iter()
            .map(|project| {
                format!(
                    "{} - {} | {}",
                    project.name, project.description, project.date_range
                )
            })
            .collect(),
    ));

    // Experience
    out.push(Element::Title("EXPERIENCE".to_string()));
    for e in &s.experience {
        out.push(Element::Experience(e.clone()));
    }

    // Early career
    out.push(Element::Title("EARLY CAREER".to_string()));
    out.push(Element::Body(format!(
        "{} - {}",
        s.early_career.date_range, s.early_career.summary
    )));

    // Technical skills
    let ts = &s.technical_skills;
    out.push(Element::Title("TECHNICAL SKILLS".to_string()));
    out.push(Element::Table(vec![
        ("Languages".to_string(), ts.languages.join(", ")),
        ("Frameworks".to_string(), ts.frameworks.join(", ")),
        ("AI/ML".to_string(), ts.ai_ml_skills.join(", ")),
        (
            "Backend/Cloud".to_string(),
            ts.backend_cloud_skills.join(", "),
        ),
        ("Blockchain".to_string(), ts.blockchain_skills.join(", ")),
        ("Mobile".to_string(), ts.mobile_skills.join(", ")),
        ("Tools".to_string(), ts.tools.join(", ")),
    ]));

    // Languages
    out.push(Element::Title("LANGUAGES".to_string()));
    out.push(Element::List(
        s.languages
            .iter()
            .map(|l| format!("{}: {}", l.name, l.proficiency))
            .collect(),
    ));

    // Education
    out.push(Element::Title("EDUCATION".to_string()));
    for e in &s.education {
        out.push(Element::Education(e.clone()));
    }

    out
}
