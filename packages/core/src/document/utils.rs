//! The document, in order.
//!
//! Reordering, retitling or removing a section is an edit to `build_sections`;
//! no rendering code changes. A section with no data returns nothing and takes
//! its heading with it.
//!
//! Reordering, retitling or removing a section is an edit to `build_sections`;
//! no rendering code changes. A section whose data is empty returns `None` and
//! takes its heading with it.

use crate::schema::types::{
    CVEducation, CVExperience, CVLinks, CVPerson, CVSchema, CVTechnicalSkills,
};

use crate::document::types::{Block, Entry, EntryVariant, LabelValueRow, ProseVariant, Section};

/// Strip the scheme and `www.` from a link, matching the reference's
/// `normalizeLink`.
pub fn normalize_link(value: &str) -> String {
    let trimmed = value
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    trimmed.trim_start_matches("www.").to_string()
}

fn format_github(value: &str) -> String {
    let normalized = normalize_link(value);
    if normalized.starts_with("github.com/") {
        format!("GitHub: {normalized}")
    } else {
        format!("GitHub: github.com/{normalized}")
    }
}

fn format_linkedin(value: &str) -> String {
    let normalized = normalize_link(value);
    if normalized.starts_with("linkedin.com/") {
        format!("LinkedIn: {normalized}")
    } else {
        let handle = normalized.trim_start_matches("in/");
        format!("LinkedIn: linkedin.com/{handle}")
    }
}

fn format_portfolio(value: &str) -> String {
    format!("Portfolio: {}", normalize_link(value))
}

/// The contact items, in reference order, with empties dropped.
pub fn contact_items(person: &CVPerson, links: &CVLinks) -> Vec<String> {
    let mut items = vec![person.location.clone()];

    if !person.email.trim().is_empty() {
        items.push(format!("Email: {}", person.email));
    }
    // The public schema carries no phone at all; an unconditional line here
    // would render "Phone: " with nothing after it.
    if let Some(phone) = person.phone.as_ref().filter(|p| !p.trim().is_empty()) {
        items.push(format!("Phone: {phone}"));
    }
    if !links.github.trim().is_empty() {
        items.push(format_github(&links.github));
    }
    if !links.linkedin.trim().is_empty() {
        items.push(format_linkedin(&links.linkedin));
    }
    if !links.portfolio.trim().is_empty() {
        items.push(format_portfolio(&links.portfolio));
    }

    items.retain(|item| !item.trim().is_empty());
    items
}

/// Split a profile-style string into paragraphs on blank lines, matching the
/// reference's `profile.split(/\n\s*\n/)`.
pub fn paragraphs(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                out.push(current.join(" "));
                current.clear();
            }
        } else {
            current.push(line.trim());
        }
    }
    if !current.is_empty() {
        out.push(current.join(" "));
    }

    out.into_iter().filter(|p| !p.trim().is_empty()).collect()
}

const SEPARATOR: &str = " \u{00b7} ";
const EM_DASH: &str = "\u{2014}";
const EN_DASH: &str = "\u{2013}";
const MIDDOT: &str = "\u{00b7}";

/// "2026-04" renders as "2026"; anything else passes through, e.g. "Present".
fn format_date(date: &str) -> String {
    let bytes = date.as_bytes();
    let is_year_month = bytes.len() == 7
        && bytes[4] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..].iter().all(u8::is_ascii_digit);

    if is_year_month {
        date[..4].to_string()
    } else {
        date.to_string()
    }
}

fn skill_rows(skills: &CVTechnicalSkills) -> Vec<LabelValueRow> {
    let categories: [(&str, &Vec<String>); 7] = [
        ("Languages", &skills.languages),
        ("Frameworks", &skills.frameworks),
        ("AI / Machine Learning", &skills.ai_ml_skills),
        ("Blockchain / Web3", &skills.blockchain_skills),
        ("Mobile", &skills.mobile_skills),
        ("Backend / Cloud", &skills.backend_cloud_skills),
        ("Tools", &skills.tools),
    ];

    categories
        .into_iter()
        .map(|(label, items)| LabelValueRow {
            label: label.to_string(),
            value: items.join(", "),
        })
        .filter(|row| !row.value.is_empty())
        .collect()
}

fn experience_entry(job: &CVExperience) -> Entry {
    Entry {
        title: job.role.clone(),
        meta: vec![format!(
            "{} {EM_DASH} {} {MIDDOT} {}{EN_DASH}{}",
            job.company,
            job.location,
            format_date(&job.start_date),
            format_date(&job.end_date),
        )],
        summary: Some(job.summary.clone()).filter(|s| !s.trim().is_empty()),
        bullets: job.highlights.clone(),
    }
}

fn education_entry(edu: &CVEducation) -> Entry {
    Entry {
        title: edu.institution.clone(),
        meta: vec![
            format!("{} {EM_DASH} {}", edu.degree, edu.field),
            format!(
                "{} {MIDDOT} {}{EN_DASH}{}",
                edu.location, edu.start_date, edu.end_date
            ),
        ],
        summary: None,
        bullets: edu.honors.clone(),
    }
}

pub fn build_sections(schema: &CVSchema) -> Vec<Section> {
    let mut sections = Vec::new();

    // Profile
    let profile = paragraphs(&schema.profile);
    if !profile.is_empty() {
        sections.push(Section::new(
            "Profile",
            Block::Prose {
                paragraphs: profile,
                variant: ProseVariant::Lead,
            },
        ));
    }

    // Core Competencies
    if !schema.core_competencies.is_empty() {
        sections.push(Section::new(
            "Core Competencies",
            Block::InlineList {
                items: schema.core_competencies.clone(),
                separator: SEPARATOR.to_string(),
                rows: 2,
            },
        ));
    }

    // Specialization Focus
    if !schema.technical_focus_areas.is_empty() {
        sections.push(Section::new(
            "Specialization Focus",
            Block::InlineList {
                items: schema.technical_focus_areas.clone(),
                separator: SEPARATOR.to_string(),
                rows: 1,
            },
        ));
    }

    // Key Achievements
    if !schema.key_achievements.is_empty() {
        sections.push(Section::new(
            "Key Achievements",
            Block::BulletList {
                items: schema.key_achievements.clone(),
                trailing_spacer: true,
            },
        ));
    }

    // Tech Leadership
    if !schema.tech_leadership.is_empty() {
        sections.push(
            Section::new(
                "Tech Leadership",
                Block::BulletList {
                    items: schema.tech_leadership.clone(),
                    trailing_spacer: false,
                },
            )
            .with_margin_top(4.0),
        );
    }

    // Selected Projects
    if !schema.selected_projects.is_empty() {
        sections.push(Section::new(
            "Selected Projects",
            Block::BulletList {
                items: schema
                    .selected_projects
                    .iter()
                    .map(|p| {
                        format!(
                            "{} {EM_DASH} {} {MIDDOT} {}",
                            p.name, p.description, p.date_range
                        )
                    })
                    .collect(),
                trailing_spacer: false,
            },
        ));
    }

    // Experience
    if !schema.experience.is_empty() {
        sections.push(Section::new(
            "Experience",
            Block::EntryList {
                entries: schema.experience.iter().map(experience_entry).collect(),
                variant: EntryVariant::Ruled,
                wrap: true,
            },
        ));
    }

    // Early Career
    if let Some(early) = &schema.early_career {
        sections.push(Section::new(
            "Early Career",
            Block::Prose {
                paragraphs: vec![format!("{} {MIDDOT} {}", early.date_range, early.summary)],
                variant: ProseVariant::Body,
            },
        ));
    }

    // Technical Skills
    let rows = skill_rows(&schema.technical_skills);
    if !rows.is_empty() {
        sections.push(
            Section::new("Technical Skills", Block::LabelValue { rows }).with_margin_top(8.0),
        );
    }

    // Languages
    if !schema.languages.is_empty() {
        sections.push(Section::new(
            "Languages",
            Block::Prose {
                paragraphs: vec![
                    schema
                        .languages
                        .iter()
                        .map(|l| format!("{} ({})", l.name, l.proficiency))
                        .collect::<Vec<_>>()
                        .join(", "),
                ],
                variant: ProseVariant::Body,
            },
        ));
    }

    // Education
    if !schema.education.is_empty() {
        sections.push(Section::new(
            "Education",
            Block::EntryList {
                entries: schema.education.iter().map(education_entry).collect(),
                variant: EntryVariant::Plain,
                wrap: false,
            },
        ));
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{CVLanguage, CVLinks, CVPerson};

    fn person() -> CVPerson {
        CVPerson {
            name: "Ada Lovelace".into(),
            location: "London, UK".into(),
            email: "ada@example.com".into(),
            phone: Some("+44 20 7946 0000".into()),
        }
    }

    #[test]
    fn normalises_link_schemes() {
        assert_eq!(normalize_link("https://github.com/x"), "github.com/x");
        assert_eq!(normalize_link("http://www.example.com"), "example.com");
        assert_eq!(normalize_link("www.example.com"), "example.com");
    }

    #[test]
    fn formats_links_without_doubling_the_host() {
        assert_eq!(
            format_github("https://github.com/x"),
            "GitHub: github.com/x"
        );
        assert_eq!(format_github("x"), "GitHub: github.com/x");
        assert_eq!(
            format_linkedin("www.linkedin.com/in/x"),
            "LinkedIn: linkedin.com/in/x"
        );
        assert_eq!(format_linkedin("in/x"), "LinkedIn: linkedin.com/x");
        assert_eq!(format_portfolio("https://x.dev"), "Portfolio: x.dev");
    }

    #[test]
    fn drops_empty_contact_fields() {
        let links = CVLinks {
            github: "https://github.com/x".into(),
            linkedin: String::new(),
            portfolio: "   ".into(),
        };
        let items = contact_items(&person(), &links);
        assert_eq!(
            items,
            vec![
                "London, UK",
                "Email: ada@example.com",
                "Phone: +44 20 7946 0000",
                "GitHub: github.com/x",
            ]
        );
    }

    #[test]
    fn omits_the_phone_line_when_there_is_no_phone() {
        let mut without = person();
        without.phone = None;
        let items = contact_items(&without, &CVLinks::default());
        assert!(
            !items.iter().any(|item| item.starts_with("Phone")),
            "a public CV must not render an empty phone line: {items:?}"
        );
    }

    #[test]
    fn splits_paragraphs_on_blank_lines() {
        assert_eq!(
            paragraphs("one\n\ntwo\n\n\nthree"),
            vec!["one", "two", "three"]
        );
        assert_eq!(
            paragraphs("wrapped\nacross lines"),
            vec!["wrapped across lines"]
        );
        assert!(paragraphs("   \n\n  ").is_empty());
    }

    #[test]
    fn shortens_year_month_dates_only() {
        assert_eq!(format_date("2026-04"), "2026");
        assert_eq!(format_date("Present"), "Present");
        assert_eq!(format_date("2019"), "2019");
        assert_eq!(format_date("2019-1"), "2019-1");
        assert_eq!(format_date("abcd-ef"), "abcd-ef");
    }

    #[test]
    fn drops_skill_categories_with_no_entries() {
        let skills = CVTechnicalSkills {
            languages: vec!["Rust".into()],
            blockchain_skills: vec![],
            ..CVTechnicalSkills::default()
        };
        let rows = skill_rows(&skills);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "Languages");
    }

    #[test]
    fn an_empty_schema_produces_no_sections() {
        assert!(build_sections(&CVSchema::default()).is_empty());
    }

    #[test]
    fn sections_appear_in_reference_order() {
        let schema = CVSchema {
            profile: "A profile.".into(),
            core_competencies: vec!["One".into()],
            languages: vec![CVLanguage {
                name: "English".into(),
                proficiency: "Native".into(),
            }],
            ..CVSchema::default()
        };
        let sections = build_sections(&schema);
        let titles: Vec<&str> = sections.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(titles, vec!["Profile", "Core Competencies", "Languages"]);
    }

    #[test]
    fn experience_meta_uses_reference_separators_and_short_dates() {
        let job = CVExperience {
            company: "Acme".into(),
            role: "Engineer".into(),
            location: "Berlin".into(),
            start_date: "2020-01".into(),
            end_date: "Present".into(),
            ..CVExperience::default()
        };
        let entry = experience_entry(&job);
        assert_eq!(entry.title, "Engineer");
        assert_eq!(
            entry.meta[0],
            "Acme \u{2014} Berlin \u{00b7} 2020\u{2013}Present"
        );
        assert!(entry.summary.is_none(), "an empty summary is omitted");
    }

    #[test]
    fn education_carries_honors_as_bullets() {
        let edu = CVEducation {
            institution: "Example University".into(),
            degree: "BSc".into(),
            field: "Mathematics".into(),
            location: "London".into(),
            start_date: "1830".into(),
            end_date: "1833".into(),
            honors: vec!["Distinction".into()],
        };
        let entry = education_entry(&edu);
        assert_eq!(entry.meta.len(), 2);
        assert_eq!(entry.bullets, vec!["Distinction"]);
    }
}
