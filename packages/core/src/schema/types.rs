#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CVPerson {
    pub name: String,
    pub location: String,
    pub email: String,
    pub phone: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CVLinks {
    pub github: String,
    pub linkedin: String,
    pub portfolio: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CVProject {
    pub name: String,
    pub description: String,
    pub date_range: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CVEarlyCareer {
    pub date_range: String,
    pub summary: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CVExperience {
    pub company: String,
    pub role: String,
    pub location: String,
    pub start_date: String,
    pub end_date: String,
    pub summary: String,
    pub highlights: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CVTechnicalSkills {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub ai_ml_skills: Vec<String>,
    pub backend_cloud_skills: Vec<String>,
    pub blockchain_skills: Vec<String>,
    pub mobile_skills: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CVLanguage {
    pub name: String,
    pub proficiency: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CVEducation {
    pub institution: String,
    pub degree: String,
    pub field: String,
    pub location: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct CVSchema {
    pub person: CVPerson,
    pub links: CVLinks,
    pub profile: String,
    pub core_competencies: Vec<String>,
    pub technical_focus_areas: Vec<String>,
    pub key_achievements: Vec<String>,
    pub tech_leadership: Vec<String>,
    pub selected_projects: Vec<CVProject>,
    pub early_career: CVEarlyCareer,
    pub experience: Vec<CVExperience>,
    pub technical_skills: CVTechnicalSkills,
    pub languages: Vec<CVLanguage>,
    pub education: Vec<CVEducation>,
}
