extern crate self as vector_db;
use std::error::Error;
use std::sync::{Arc, Mutex};

use arrow_array::RecordBatchIterator;
use fastembed::{EmbeddingModel, InitOptionsWithLength, TextEmbedding};
use lancedb::Table;
use lancedb::{Connection, arrow::arrow_schema::FieldRef};
use marrow::datatypes::{DataType, Field};
use serde::{Deserialize, Serialize};
pub use serde_arrow; // ::schema::{SchemaLike, TracingOptions};
pub use vector_db_macro::VectorDbRecord;

#[derive(VectorDbRecord)]
pub struct Chunk {
    id: String,
    #[vector]
    text: String,
}

#[derive(Clone)]
pub struct VectorDb {
    conn: Connection,
    pub model: Arc<Mutex<TextEmbedding>>,
}

pub trait Embeddable {
    type Item;

    fn embed(&self, model: &mut TextEmbedding) -> Self::Item;

    fn tracing_options(&self) -> serde_arrow::schema::TracingOptions;
}

impl VectorDb {
    pub async fn try_new(path: &str) -> Result<Self, Box<dyn Error>> {
        let conn = lancedb::connect(path).execute().await?;
        let options =
            InitOptionsWithLength::new(EmbeddingModel::BGESmallENV15).with_max_length(384);
        let model = Arc::new(Mutex::new(TextEmbedding::try_new(options).unwrap()));
        Ok(Self { conn, model })
    }

    pub async fn create_table_with_data<T>(
        &mut self,
        name: &str,
        data: Vec<impl Embeddable<Item = T>>,
    ) -> Result<(), Box<dyn Error>>
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        use serde_arrow::schema::SchemaLike;

        let mut model = self.model.lock().expect("model exclusive lock obtained");
        let xs: Vec<T> = data.iter().map(|d| d.embed(&mut model)).collect();
        let Some(opts) = data.iter().peekable().peek().map(|e| e.tracing_options()) else {
            return Ok(());
        };
        let fields = Vec::<FieldRef>::from_type::<T>(opts)?;
        let batch = serde_arrow::to_record_batch(&fields, &xs)?;
        let schema = batch.schema();
        let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema.clone());
        self.conn
            .drop_all_tables(&[])
            .await
            .expect("dropped all tables"); // for simplicity
        self.conn.create_table(name, batches).execute().await?;
        Ok(())
    }

    pub async fn open(&self, table_name: impl Into<String>) -> Result<Table, Box<dyn Error>> {
        let table = self.conn.open_table(table_name).execute().await?;
        Ok(table)
    }
}

pub fn fixed_size_list_field(name: impl Into<String>, size: u16) -> Field {
    Field {
        name: name.into(),
        data_type: DataType::FixedSizeList(
            Box::new(Field {
                name: "item".into(),
                data_type: DataType::Float32,
                nullable: false,
                metadata: std::collections::HashMap::new(),
            }),
            size as i32,
        ),
        nullable: false,
        metadata: std::collections::HashMap::new(),
    }
}
