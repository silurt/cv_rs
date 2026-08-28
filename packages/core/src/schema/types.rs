//! The CV data model.
//!
//! Almost every field is optional. A section whose data is empty removes itself
//! from the document, heading included — that is how the reference behaves, and
//! it is why the collections carry `#[serde(default)]` rather than being required.

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct CVPerson {
    pub name: String,
    pub location: String,
    pub email: String,
    /// Absent on a public CV, which carries no phone number.
    pub phone: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct CVLinks {
    pub github: String,
    pub linkedin: String,
    pub portfolio: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct CVProject {
    pub name: String,
    pub description: String,
    pub date_range: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct CVEarlyCareer {
    pub date_range: String,
    pub summary: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct CVExperience {
    pub company: String,
    pub role: String,
    pub location: String,
    pub start_date: String,
    pub end_date: String,
    pub summary: String,
    pub highlights: Vec<String>,
    /// Carried in the data but deliberately not rendered, matching the reference.
    pub tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct CVTechnicalSkills {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub ai_ml_skills: Vec<String>,
    pub blockchain_skills: Vec<String>,
    pub mobile_skills: Vec<String>,
    pub backend_cloud_skills: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct CVLanguage {
    pub name: String,
    pub proficiency: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct CVEducation {
    pub institution: String,
    pub degree: String,
    pub field: String,
    pub location: String,
    pub start_date: String,
    pub end_date: String,
    pub honors: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(default)]
pub struct CVSchema {
    pub person: CVPerson,
    pub links: CVLinks,
    pub profile: String,
    pub core_competencies: Vec<String>,
    pub technical_focus_areas: Vec<String>,
    pub key_achievements: Vec<String>,
    pub tech_leadership: Vec<String>,
    pub selected_projects: Vec<CVProject>,
    pub early_career: Option<CVEarlyCareer>,
    pub experience: Vec<CVExperience>,
    pub technical_skills: CVTechnicalSkills,
    pub languages: Vec<CVLanguage>,
    pub education: Vec<CVEducation>,
}
