//! End-to-end checks against the committed example schema.

use core::schema::types::CVSchema;

fn example() -> CVSchema {
    let json = include_str!("../../../examples/example.json");
    serde_json::from_str(json).expect("examples/example.json should match CVSchema")
}

#[test]
fn renders_the_example_schema() {
    let mut doc = render::render(&example()).expect("render should succeed");
    let bytes = doc.to_bytes().expect("document should serialise");
    assert!(bytes.starts_with(b"%PDF"), "output should be a PDF");
    assert!(
        bytes.len() > 2_000,
        "output looks empty: {} bytes",
        bytes.len()
    );
}

#[test]
fn renders_an_empty_schema_without_panicking() {
    let mut doc = render::render(&CVSchema::default()).expect("render should succeed");
    assert!(doc.to_bytes().is_ok());
}

#[test]
fn a_schema_with_no_phone_still_renders() {
    let mut schema = example();
    schema.person.phone = None;
    let mut doc = render::render(&schema).expect("render should succeed");
    assert!(doc.to_bytes().is_ok());
}
