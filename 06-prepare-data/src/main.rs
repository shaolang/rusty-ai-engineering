use transmutation::Converter;

#[tokio::main]
async fn main() {
    // flossmanual.net doesn't seem to be working anymore
    let source = "https://raw.githubusercontent.com/jaywengrow/floss-manuals/main/firefox.pdf";
    let pdf_fname = "target/firefox.pdf";
    let markdown_fname = "target/flamehamster.md";
    download_file(source, pdf_fname).await;
    convert(pdf_fname, markdown_fname).await.unwrap();
}

async fn download_file(url: &str, fname: &str) {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::File::create(fname).await.expect("file created");
    let mut bytes = reqwest::get(url).await.expect("downloaded").bytes_stream();
    while let Some(chunk) = bytes.next().await {
        if let Ok(raw) = chunk {
            file.write_all(&raw).await.expect("chunk written");
        }
    }
}

async fn convert(pdf_fname: &str, markdown_fname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let converter = Converter::new()?;
    let result = converter
        .convert(pdf_fname)
        .to(transmutation::OutputFormat::Markdown {
            split_pages: false,
            optimize_for_llm: true,
        })
        .execute()
        .await?;
    result.save(markdown_fname).await?;

    Ok(())
}
