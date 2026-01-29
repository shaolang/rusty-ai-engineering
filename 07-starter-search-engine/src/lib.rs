use std::fs;

use fastembed::TextEmbedding;
use pcre2::bytes::RegexBuilder;

pub fn create_embeddings_by_h1_headers_from_markdown_text(
    filename: &str,
    manual_name: &str,
) -> Vec<Record> {
    let md_text = fs::read_to_string(filename).expect("read markdown file");
    let chunks = split_markdown_by_h1(md_text);

    let mut embedder = TextEmbedder::new();
    chunks
        .iter()
        .enumerate()
        .map(|(i, s)| embedder.create_record(i, s, manual_name))
        .collect()
}

fn split_markdown_by_h1(md_text: impl AsRef<str>) -> Vec<String> {
    RegexBuilder::new()
        .dotall(true)
        .crlf(true)
        .build(r#"(?m)^# .+?(?=^# |\Z)"#)
        .expect("well-formed regex")
        .find_iter(&md_text.as_ref().as_bytes())
        .filter_map(|m| m.map(|m| m.as_bytes()).ok())
        .map(|m| String::from_utf8_lossy(m).trim().to_string())
        .collect()
}

#[derive(Debug)]
pub struct Record {
    chunk_id: String,
    chunk_text: String,
    manual: String,
    vector: Vec<f32>,
}

struct TextEmbedder {
    model: TextEmbedding,
}

impl TextEmbedder {
    fn new() -> Self {
        let model = TextEmbedding::try_new(Default::default()).expect("text embedding created");
        Self { model }
    }

    fn create_record(
        &mut self,
        id: usize,
        text: impl AsRef<str>,
        manual: impl AsRef<str>,
    ) -> Record {
        let vector = self
            .model
            .embed([text.as_ref()], None)
            .map(|vv| vv[0].to_owned())
            .expect("created embeddings");

        Record {
            chunk_id: format!("chunk-{id:05}"),
            chunk_text: text.as_ref().to_string(),
            manual: manual.as_ref().to_string(),
            vector,
        }
    }
}
