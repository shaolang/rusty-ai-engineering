use std::{fs, sync::Arc};

use arrow_array::builder::{Float32Builder, ListBuilder};
use arrow_array::{ArrayRef, RecordBatch, StringArray};
use fastembed::TextEmbedding;
use pcre2::bytes::RegexBuilder;

use crate::IntoRecordBatches;

#[derive(Debug)]
pub struct Chunk {
    chunk_id: String,
    chunk_text: String,
    manual: String,
    vector: Vec<f32>,
}

#[derive(Debug)]
pub struct Document {
    chunks: Vec<Chunk>,
}

struct TextEmbedder {
    model: TextEmbedding,
}

pub fn chunk_markdown_text_by_h1_header(filename: &str, manual_name: &str) -> Document {
    let md_text = fs::read_to_string(filename).expect("read markdown file");
    let text_chunks = split_markdown_by_h1(md_text);

    let mut embedder = TextEmbedder::new();
    let chunks = text_chunks
        .iter()
        .enumerate()
        .map(|(i, s)| embedder.create_record(i, s, manual_name))
        .collect();

    Document { chunks }
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

impl IntoRecordBatches for Document {
    fn into(self) -> RecordBatch {
        let ids = StringArray::from_iter_values(self.chunks.iter().map(|c| c.chunk_id.as_str()));
        let texts =
            StringArray::from_iter_values(self.chunks.iter().map(|c| c.chunk_text.as_str()));
        let manuals = StringArray::from_iter_values(self.chunks.iter().map(|c| c.manual.as_str()));
        let mut builder = ListBuilder::new(Float32Builder::new());
        self.chunks.iter().for_each(|chunk| {
            builder.values().append_slice(&chunk.vector);
            builder.append(true);
        });
        let vectors = builder.finish();

        RecordBatch::try_from_iter(vec![
            ("chunk_id", Arc::new(ids) as ArrayRef),
            ("chunk_text", Arc::new(texts) as ArrayRef),
            ("manual", Arc::new(manuals) as ArrayRef),
            ("vector", Arc::new(vectors) as ArrayRef),
        ])
        .expect("created record batch")
    }
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
    ) -> Chunk {
        let vector = self
            .model
            .embed([text.as_ref()], None)
            .map(|vv| vv[0].to_owned())
            .expect("created embeddings");

        Chunk {
            chunk_id: format!("chunk-{id:05}"),
            chunk_text: text.as_ref().to_string(),
            manual: manual.as_ref().to_string(),
            vector,
        }
    }
}
