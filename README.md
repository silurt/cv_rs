# cv_rs

Rust workspace for generating CV/resume PDFs.

The output is built to match the reference React renderer (`../cv`, `@react-pdf/renderer`)
as closely as the format allows — see [Parity](#parity).

## Commands

```bash
cargo build                   # build all packages
cargo test                    # run all tests
cargo clippy --all-targets    # lint
cargo fmt --all               # format

cargo run -p cli -- --schema-path examples/example.json --output-path output/cv.pdf
```

## CLI

```
Usage: cv --schema-path <SCHEMA_PATH> --output-path <OUTPUT_PATH>

Options:
  -s, --schema-path <SCHEMA_PATH>  path to the JSON schema file
  -o, --output-path <OUTPUT_PATH>  path to write the output PDF (directories are created)
  -h, --help                       Print help
  -V, --version                    Print version
```

`examples/example.json` is a complete, fictional CV covering every section. Keep real
CV data out of the repository — `output/` is gitignored for exactly this reason.

## Workspace architecture

```
packages/
  core/    — the data model (CVSchema and friends)
  render/  — layout and PDF rendering
  cli/     — binary `cv` that wires core + render, writes output
```

`render` is layered, mirroring the reference's own structure:

| Module | Responsibility |
|---|---|
| `style` | Every metric and colour, ported one-for-one from the reference stylesheet, plus the Helvetica AFM tables in `style::metrics` |
| `layout` | Text measurement, line breaking (`layout::linebreak`), and the page/cursor machine (`layout::types::Renderer`) |
| `block` | The five block shapes a section can render, plus the header |
| `sections` | Maps a `CVSchema` to an ordered list of sections |

**Data flow**: `CVSchema` → `sections::build_sections` → `Vec<Section>` → `block::render_section`
→ `Renderer` → `oxidize_pdf::Document` → `doc.save()`.

### Blocks

A section renders exactly one block. Adding a shape means adding a variant to `Block`
and a match arm in `block::utils::render_block`; nothing else changes.

- `Prose` — paragraphs of running text (`Lead` is the larger, justified profile treatment)
- `InlineList` — items joined onto one or more balanced lines, e.g. "A · B · C"
- `BulletList` — a flat list of bulleted lines
- `EntryList` — repeated title/meta/summary/bullets records (`Ruled` for Experience, `Plain` for Education)
- `LabelValue` — two-column rows, used by the skills table

### Layout

`Renderer` works in CSS coordinates — the cursor is the distance from the top of the
page — and converts to PDF's bottom-left origin only when drawing, so every metric in
the crate is directly comparable with the reference stylesheet.

Two details are load-bearing:

- **Line breaking** is the Knuth & Plass algorithm (`layout::linebreak`), not greedy
  wrapping. The reference treats inter-word spaces as elastic glue that can shrink, so
  it will keep a word on a line that overruns the column by a fraction of a point and
  then set that line tight. Greedy wrapping breaks a word early and every subsequent
  line diverges.
- **Measurement** uses the Adobe AFM tables with kern pairs, generated from the same
  data the reference uses. Widths drive line breaking, so a systematic error here
  shifts every wrap point in the document.

## Parity

Verified against the reference render of the same data:

| Measure | Result |
|---|---|
| Pages | identical |
| Lines | 115 / 115 break identically |
| Line vertical position | within 0.2pt |
| Word horizontal position | 99.8% within 0.5pt (worst 0.59pt) |

The residual comes from justification: the reference distributes slack across both
word spaces *and* letters, while this renderer uses word spacing alone.

The layout deliberately reproduces a few quirks of the reference rather than tidying
them, because they are visible in the output:

- Kerning does not cross a text-run boundary. The reference builds a bullet line from
  two JSX children, so `"\u{2022} "` is its own run and the `space`+`W` pair does not kern.
- Where the header's contact row wraps, one space migrates from the line's first
  separator to its end, leaving the overall advance — and so the centring — unchanged.
- A line's leading sits below its baseline, so a taller line pushes the *following*
  line down rather than shifting its own glyphs.

`experience.tags` is carried in the schema but not rendered, matching the reference.

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
| `technical_skills` | `CVTechnicalSkills` | Two-column table; empty categories are dropped |
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
alone; anything else passes through, so `"Present"` works.
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

## Regenerating font metrics

`packages/render/src/style/metrics.rs` is generated, not hand-written. It comes from
the AFM tables bundled with the reference renderer, so run the generator from a
checkout of that repo (which has the `@react-pdf/pdfkit` dependency):

```bash
node ../cv_rs/scripts/gen-metrics.mjs ../cv_rs/packages/render/src/style/metrics.rs
```

Only needed if the reference changes font.
