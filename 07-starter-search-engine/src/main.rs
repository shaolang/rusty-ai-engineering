use starter_search_engine as sse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let doc = sse::chunk_markdown_text_by_h1_header("resources/flamehamster.md", "flamehamster");
    let vector_db = sse::VectorDb::connect("data/starter").await;
    vector_db.create_table("flamehamster", doc).await;

    Ok(())
}
