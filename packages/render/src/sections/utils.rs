//! The document, in order — a port of the reference `cvSections.ts`.
//!
//! Reordering, retitling or removing a section is an edit to `build_sections`;
//! no rendering code changes. A section whose data is empty returns `None` and
//! takes its heading with it.

use core::schema::types::{CVEducation, CVExperience, CVSchema, CVTechnicalSkills};

use crate::block::types::{Block, Entry, EntryVariant, LabelValueRow, ProseVariant, Section};
use crate::layout::utils::paragraphs;
use crate::style::types::{TECH_LEADERSHIP_MARGIN_TOP, TECHNICAL_SKILLS_MARGIN_TOP};

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
            .with_margin_top(TECH_LEADERSHIP_MARGIN_TOP),
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
            Section::new("Technical Skills", Block::LabelValue { rows })
                .with_margin_top(TECHNICAL_SKILLS_MARGIN_TOP),
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
    use core::schema::types::CVLanguage;

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
