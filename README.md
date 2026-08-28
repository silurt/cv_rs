# cv_rs

Generates a CV as a PDF or a Word document from a single JSON file.

The output is deliberately plain: one text column, real text, no tables or graphics,
and a tagged structure tree — so it reads well and parses cleanly. See
[Machine readability](#machine-readability).

## Commands

```bash
cargo build                   # build all packages
cargo test                    # run all tests
cargo clippy --all-targets    # lint
cargo fmt --all               # format

# The output format follows the extension.
cargo run -p cli -- --schema-path examples/example.json --output-path output/cv.pdf
cargo run -p cli -- --schema-path examples/example.json --output-path output/cv.docx
```

## CLI

```
Usage: cv [OPTIONS] --schema-path <SCHEMA_PATH> --output-path <OUTPUT_PATH>

Options:
  -s, --schema-path <SCHEMA_PATH>  path to the JSON schema file
  -o, --output-path <OUTPUT_PATH>  path to write to; .pdf or .docx (directories are created)
      --compress                   PDF only: compress content streams
      --no-tags                    PDF only: omit the Tagged PDF structure tree
  -h, --help                       Print help
  -V, --version                    Print version
```

`examples/example.json` is a complete, fictional CV covering every section. Keep real
CV data out of the repository — `output/` is gitignored for exactly this reason.

### Output modes

The PDF defaults to being *readable*. Content streams are left uncompressed, so the
page instructions are plain text you can open in an editor and follow:

```
BT
/Helvetica 10 Tf
0.333 0.333 0.333 rg
65.26 765.42 Td
(Amsterdam, NL) Tj
ET
```

| Mode | Size | Notes |
|---|---|---|
| default | ~108 KB | readable stream, full structure tree |
| `--compress` | ~41 KB | same document, streams deflated |
| `--no-tags` | ~86 KB | readable, no structure tree |
| `--no-tags --compress` | ~19 KB | smallest |
| `.docx` | ~6 KB | always deflated; a `.docx` is a zip |

Every mode carries identical text and identical layout. File size has no bearing on
how a CV is read or parsed — pick whichever you want to live with.

## Machine readability

**The best thing you can do for a CV is have the experience.** Nothing here is a
substitute for that, and no formatting choice will make a weaker candidate beat a
stronger one.

What a clean document does is narrower, and worth being precise about: it makes sure a
parser actually *sees* the experience you already have. Parsing failures are
subtractive — a job title swallowed by a table cell, or contact details sitting in a
page header, are not scored badly, they are simply absent from the record a recruiter
searches. Between two candidates with comparable backgrounds, the one whose CV parses
cleanly is the one who doesn't quietly lose fields on the way in.

One caveat worth stating plainly, because most writing on this subject is marketing
from companies selling resume scanners: an "ATS score" is largely a vendor construct.
Across Workday, Greenhouse, Lever and Ashby the default candidate list is ordered by
application date and pipeline stage, not by a hidden ranking number, and the AI
matching features that do exist surface recommendations to a human rather than
[auto-rejecting anyone](https://atsverification.com/blog/how-recruiters-sort-candidates-ats-2026/).
The goal is not to game a score. It is to be losslessly readable.

### What the output does, and why

| Property | Why it matters | How |
|---|---|---|
| Real text, never an image | Image-only and scanned PDFs are the consistent failure case across every major ATS, because parsers look for a text layer and [do not run OCR](https://www.loopcv.pro/guides/ats-resume-not-parsed/) | All content is drawn as text operators with standard fonts; the only graphics are two hairline rules |
| Single column | Parsers read a page as one linear left-to-right, top-to-bottom stream. Multi-column templates have been measured dropping skills extraction to [as low as 46%, against 93% for single-column](https://www.jobscan.co/blog/resume-tables-columns-ats/) | Every block spans the full text column; nothing sits side by side |
| No tables for content | Table cells are consumed in [unpredictable order](https://www.jobscan.co/blog/resume-tables-columns-ats/) — sometimes by row, sometimes by column — interleaving unrelated text | The skills block *looks* like two columns but is a label/value list carrying no table structure |
| Nothing in headers or footers | Headers and footers live in a separate layer that most systems [skip entirely](https://www.jobscan.co/blog/ats-formatting-mistakes/), so contact details placed there disappear | There are none. Contact details are the first body text on page one |
| Conventional section headings | Parsers segment a document by matching known headings before extracting fields | Headings are the standard set — Profile, Experience, Education, Technical Skills — in a fixed order |
| Tagged structure tree | A tagged PDF carries logical structure independent of visual position, which is what lets text be [extracted in reading order](https://pdfa.org/resource/tagged-pdf-q-a/) rather than inferred from coordinates. It is the mechanism the [PDF/UA accessibility standard](https://www.nutrient.io/blog/what-is-pdf-ua/) (ISO 14289) is built on | On by default. Headings are `H1`/`H2`/`H3`, prose is `P`, bullets are real `L`/`LI`/`LBody` lists, skills are `Lbl`/`LBody` pairs |
| A `.docx` alternative | Several widely deployed systems were built around Word and parse `.docx` [more reliably than PDF](https://www.jobscan.co/blog/resume-pdf-vs-word/) | `--output-path cv.docx` emits the same document with real `Heading1`–`Heading3` styles |

### Structural decisions

The layout choices are mostly *absences*, which is the point — every feature a designer
would reach for is a place a parser can lose data.

**One text column, one reading order.** Visual order and logical order are the same.
No sidebar, no two-column skills grid, no floating blocks. A parser reading strictly
top-to-bottom and one walking the structure tree arrive at the same document.

**The skills block is a list, not a table.** It renders as an aligned label column and
a value column, which reads like a table, but structurally it is a list of label/value
pairs. This is the one place the visual design and the underlying structure
deliberately diverge, because a real table is the most common cause of scrambled
output.

**Bullets are list structure, not typography.** Each bullet is an `LI` containing an
`LBody`, not a paragraph that happens to start with a `•`. A structure-aware extractor
recovers a list; a naive one still sees the bullet character.

**Entries are self-contained.** Each role is a `Sect` holding a heading, a meta line
and its bullets, kept on one page where it fits. A role split across a page boundary is
a common way for dates and employers to end up attached to the wrong job.

**Dates take one predictable form.** `2020-01` renders as `2020`, ranges use an en
dash, and the meta line has a fixed shape: `Company — Location · Start–End`.
Consistency matters more than the particular format, because parsers match on shape.

**No decorative anything.** No icons, skill bars, photographs, text inside graphics, or
colour carrying meaning. The only non-text marks are two grey rules, and removing them
would change nothing that is read.

**Empty sections remove themselves.** A section with no data omits its heading too,
rather than leaving a heading for a parser to mis-associate with whatever follows.

## Workspace

```
packages/
  core/    — the data model (CVSchema) and the document model (sections, blocks)
  render/  — layout and PDF rendering
  docx/    — Word output from the same document model
  cli/     — binary `cv` that picks a backend by file extension
```

`core::document` is format-independent: it turns a `CVSchema` into an ordered list of
sections, each holding one block (`Prose`, `InlineList`, `BulletList`, `EntryList`,
`LabelValue`). Both backends consume that, so section order, the wording of every line
and date formatting are shared rather than reimplemented per format. Adding a new
shape means adding a `Block` variant and a match arm in each backend; adding a new
output format means a new package that reads the same sections.

## Schema

The CLI reads a JSON file conforming to `CVSchema`. Every field is optional and
defaults to empty; a section whose data is empty removes itself from the document,
heading included.

### Top-level fields

| Field | Type | Description |
|---|---|---|
| `person` | `CVPerson` | Name and contact details |
| `links` | `CVLinks` | Social/professional links |
| `profile` | `string` | Professional summary; blank lines separate paragraphs |
| `core_competencies` | `string[]` | Balanced across two lines |
| `technical_focus_areas` | `string[]` | Rendered as "Specialization Focus" |
| `key_achievements` | `string[]` | Bulleted |
| `tech_leadership` | `string[]` | Bulleted |
| `selected_projects` | `CVProject[]` | Bulleted |
| `early_career` | `CVEarlyCareer?` | Omitted entirely when absent |
| `experience` | `CVExperience[]` | Ruled entries |
| `technical_skills` | `CVTechnicalSkills` | Label/value rows; empty categories are dropped |
| `languages` | `CVLanguage[]` | Rendered as one line |
| `education` | `CVEducation[]` | Plain entries |

### Nested types

**CVPerson** — `phone` is optional, so a public CV can omit it entirely rather than
rendering an empty line.
```json
{ "name": "", "location": "", "email": "", "phone": "+31 6 00000000" }
```

**CVLinks** — empty values are dropped from the contact row. Schemes and `www.` are
stripped, and the host is added if missing.
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

**CVExperience** — `start_date`/`end_date` of the form `YYYY-MM` render as the year
alone; anything else passes through, so `"Present"` works. `tags` is carried but not
rendered.
```json
{
  "company": "", "role": "", "location": "",
  "start_date": "2020-01", "end_date": "Present",
  "summary": "", "highlights": [], "tags": []
}
```

**CVTechnicalSkills**
```json
{
  "languages": [], "frameworks": [], "ai_ml_skills": [],
  "blockchain_skills": [], "mobile_skills": [],
  "backend_cloud_skills": [], "tools": []
}
```

**CVLanguage**
```json
{ "name": "", "proficiency": "" }
```

**CVEducation**
```json
{
  "institution": "", "degree": "", "field": "", "location": "",
  "start_date": "", "end_date": "", "honors": []
}
```

See `examples/example.json` for a complete document.
