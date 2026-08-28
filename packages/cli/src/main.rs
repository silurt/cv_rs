use clap::Parser;
use core::schema::types::CVSchema;
use render::{RenderOptions, render_with};
use std::io::Write;
use std::path::Path;

/// Render a CV described by a JSON schema.
///
/// The output format follows the extension of `--output-path`: `.pdf` renders a
/// Tagged PDF, `.docx` renders a Word document from the same document model.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the JSON schema file.
    #[arg(short, long)]
    schema_path: String,

    /// Path to write the output to. Missing directories are created.
    #[arg(short, long)]
    output_path: String,

    /// PDF only: compress the content streams. Smaller file, but the page
    /// instructions become a binary blob. Off by default, so the PDF can be
    /// opened in a text editor and read.
    #[arg(long)]
    compress: bool,

    /// PDF only: omit the Tagged PDF structure tree. Produces a smaller file
    /// that is still fully text-extractable, but leaves a parser to infer
    /// headings and lists from layout rather than reading them.
    #[arg(long)]
    no_tags: bool,
}

fn read_schema_from_file(path: &str) -> Result<CVSchema, anyhow::Error> {
    let schema_data = std::fs::read_to_string(Path::new(path))?;
    Ok(serde_json::from_str(&schema_data)?)
}

fn write_output(path: &Path, bytes: &[u8]) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create(path)?;
    file.write_all(bytes)?;
    Ok(())
}

fn generate_from_args(args: &Args) -> Result<(), anyhow::Error> {
    let schema = read_schema_from_file(&args.schema_path)?;
    let path = Path::new(&args.output_path);

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("pdf")
        .to_ascii_lowercase();

    match extension.as_str() {
        "docx" => write_output(path, &docx::render(&schema)?),
        "pdf" => {
            let options = RenderOptions {
                tagged: !args.no_tags,
            };
            let mut document = render_with(&schema, options)?;
            let config = oxidize_pdf::writer::WriterConfig {
                compress_streams: args.compress,
                ..Default::default()
            };
            write_output(path, &document.to_bytes_with_config(config)?)
        }
        other => Err(anyhow::anyhow!(
            "unsupported output format {other:?}; use .pdf or .docx"
        )),
    }
}

fn main() -> std::process::ExitCode {
    let args = Args::parse();

    match generate_from_args(&args) {
        Ok(()) => {
            println!("Wrote {}", args.output_path);
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
