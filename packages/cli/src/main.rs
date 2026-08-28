use clap::Parser;
use core::schema::types::CVSchema;
use render::render;

/// Render a CV described by a JSON schema to a PDF.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the JSON schema file.
    #[arg(short, long)]
    schema_path: String,

    /// Path to write the output PDF to. Missing directories are created.
    #[arg(short, long)]
    output_path: String,
}

fn read_schema_from_file(path: &str) -> Result<CVSchema, anyhow::Error> {
    let path = std::path::Path::new(path);
    let schema_data = std::fs::read_to_string(path)?;
    let schema: CVSchema = serde_json::from_str(&schema_data)?;
    Ok(schema)
}

pub fn render_to_file(relative_path: &str, schema: &CVSchema) -> Result<(), anyhow::Error> {
    let path = std::path::Path::new(&relative_path);

    // if directory doesn't exist, create it
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    let mut doc = render(schema)?;

    Ok(doc.save(path)?)
}

fn generate_pdf_from_args(args: Args) -> Result<(), anyhow::Error> {
    let schema = read_schema_from_file(&args.schema_path)?;
    render_to_file(&args.output_path, &schema)
}

fn main() -> std::process::ExitCode {
    let args = Args::parse();
    let output = args.output_path.clone();

    match generate_pdf_from_args(args) {
        Ok(()) => {
            println!("Wrote {output}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error generating PDF: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
