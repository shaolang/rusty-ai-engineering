extern crate self as vector_db;
use std::error::Error;
use std::sync::Arc;

use arrow_array::{RecordBatch, RecordBatchIterator};
use fastembed::{EmbeddingModel, InitOptionsWithLength, TextEmbedding};
use futures::TryStreamExt;
use lancedb::Table;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{Connection, arrow::arrow_schema::FieldRef};
use marrow::datatypes::{DataType, Field};
use serde::{Deserialize, Serialize};
pub use serde_arrow;
use tokio::sync::Mutex;
pub use vector_db_macro::VectorDbRecord;

#[derive(Clone)]
pub struct VectorDb {
    conn: Connection,
    pub model: Arc<Mutex<TextEmbedding>>,
}

pub struct VectorTable {
    model: Arc<Mutex<TextEmbedding>>,
    table: Table,
}

pub trait Embeddable {
    type Item: Serialize + for<'de> Deserialize<'de>;

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

        let mut model = self.model.lock().await;
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

    pub async fn open(&self, table_name: impl Into<String>) -> Result<VectorTable, Box<dyn Error>> {
        let table = self.conn.open_table(table_name).execute().await?;
        Ok(VectorTable {
            model: self.model.clone(),
            table,
        })
    }
}

impl VectorTable {
    pub async fn get_relevant_records(
        &self,
        query: impl AsRef<str>,
        target_column: impl AsRef<str>,
        top_k: usize,
    ) -> Vec<RecordBatch> {
        let embedded_query = self
            .model
            .lock()
            .await
            .embed([query.as_ref()], None)
            .map(|v| v[0].to_owned())
            .expect("query embedding created");
        self.table
            .query()
            .nearest_to(embedded_query.as_slice())
            .expect("embedded query created")
            .limit(top_k)
            .refine_factor(5)
            .nprobes(10)
            .column(target_column.as_ref())
            .execute()
            .await
            .expect("ran query against vector db")
            .try_collect()
            .await
            .unwrap()
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
