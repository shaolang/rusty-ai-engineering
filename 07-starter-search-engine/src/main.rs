use starter_search_engine as sse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let records = sse::create_embeddings_by_h1_headers_from_markdown_text(
        "resources/flamehamster.md",
        "flamehamster",
    );
    println!("{:#?}", records.last());

    Ok(())
}
