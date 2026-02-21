# cv_rs

Rust workspace for generating CV/resume PDFs.

## Commands

```bash
cargo build                   # build all packages
cargo test                    # run all tests
cargo check                   # type-check without building

# Run the CLI
cargo run -p cli -- --schema-path cv.json --output-path output/cv.pdf
```

## CLI

```
Usage: cv --schema-path <SCHEMA_PATH> --output-path <OUTPUT_PATH>

Options:
  -s, --schema-path <SCHEMA_PATH>  path to JSON schema file
  -o, --output-path <OUTPUT_PATH>  path to write the output PDF
  -h, --help                       Print help
  -V, --version                    Print version
```

## Workspace Architecture

```
packages/
  core/    — shared data types (CVSchema, CVPerson, …)
  render/  — PDF rendering logic using oxidize-pdf
  cli/     — binary `cv` that wires core + render, writes output
```

**Data flow**: `CVSchema` (core) → `render()` (render) → `Document` (oxidize-pdf) → `doc.save()` (cli)

**Key dependencies**: `oxidize-pdf` (PDF generation), `clap` (CLI args), `anyhow` (error handling), `serde` (derive)

## Schema

The CLI reads a JSON file conforming to `CVSchema`. Below is the full structure with all fields.

### Top-level fields

| Field | Type | Description |
|---|---|---|
| `person` | `CVPerson` | Name and contact details |
| `links` | `CVLinks` | Social/professional links |
| `profile` | `string` | Short professional summary |
| `core_competencies` | `string[]` | List of core skills/competencies |
| `technical_focus_areas` | `string[]` | Technical domains of expertise |
| `key_achievements` | `string[]` | Notable career achievements |
| `tech_leadership` | `string[]` | Leadership highlights |
| `selected_projects` | `CVProject[]` | Highlight projects |
| `early_career` | `CVEarlyCareer` | Summary of early career years |
| `experience` | `CVExperience[]` | Work experience entries |
| `technical_skills` | `CVTechnicalSkills` | Categorised skill lists |
| `languages` | `CVLanguage[]` | Spoken/written languages |
| `education` | `CVEducation[]` | Educational background |

### Nested types

**CVPerson**
```json
{ "name": "", "location": "", "email": "", "phone": "" }
```

**CVLinks**
```json
{ "github": "", "linkedin": "", "portfolio": "" }
```

**CVProject**
```json
{ "name": "", "description": "", "date_range": "" }
```

**CVEarlyCareer**
```json
{ "date_range": "", "summary": "" }
```

**CVExperience**
```json
{
  "company": "",
  "role": "",
  "location": "",
  "start_date": "",
  "end_date": "",
  "summary": "",
  "highlights": [],
  "tags": []
}
```

**CVTechnicalSkills**
```json
{
  "languages": [],
  "frameworks": [],
  "ai_ml_skills": [],
  "backend_cloud_skills": [],
  "blockchain_skills": [],
  "mobile_skills": [],
  "tools": []
}
```

**CVLanguage**
```json
{ "name": "", "proficiency": "" }
```

**CVEducation**
```json
{
  "institution": "",
  "degree": "",
  "field": "",
  "location": "",
  "start_date": "",
  "end_date": ""
}
```

### Minimal example

```json
{
  "person": {
    "name": "Jane Doe",
    "location": "Berlin, Germany",
    "email": "jane@example.com",
    "phone": "+49 123 456789"
  },
  "links": {
    "github": "github.com/janedoe",
    "linkedin": "linkedin.com/in/janedoe",
    "portfolio": "janedoe.dev"
  },
  "profile": "Senior software engineer with 10 years of experience.",
  "core_competencies": ["System Design", "Distributed Systems"],
  "technical_focus_areas": ["Backend", "Cloud Infrastructure"],
  "key_achievements": ["Led migration of monolith to microservices"],
  "tech_leadership": ["Managed a team of 5 engineers"],
  "selected_projects": [
    { "name": "Project X", "description": "A cool project.", "date_range": "2022–2023" }
  ],
  "early_career": {
    "date_range": "2010–2015",
    "summary": "Worked at various startups in a full-stack capacity."
  },
  "experience": [
    {
      "company": "Acme Corp",
      "role": "Senior Engineer",
      "location": "Berlin, Germany",
      "start_date": "2020-01",
      "end_date": "Present",
      "summary": "Led backend platform development.",
      "highlights": ["Reduced p99 latency by 40%"],
      "tags": ["Rust", "Kubernetes"]
    }
  ],
  "technical_skills": {
    "languages": ["Rust", "TypeScript"],
    "frameworks": ["Axum", "React"],
    "ai_ml_skills": ["PyTorch"],
    "backend_cloud_skills": ["AWS", "Kubernetes"],
    "blockchain_skills": [],
    "mobile_skills": [],
    "tools": ["Git", "Docker"]
  },
  "languages": [
    { "name": "English", "proficiency": "Native" },
    { "name": "German", "proficiency": "B2" }
  ],
  "education": [
    {
      "institution": "TU Berlin",
      "degree": "M.Sc.",
      "field": "Computer Science",
      "location": "Berlin, Germany",
      "start_date": "2008-10",
      "end_date": "2012-07"
    }
  ]
}
```
