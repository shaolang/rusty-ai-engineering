extern crate self as vector_db;
use std::{collections::HashMap, error::Error, sync::Arc};

use arrow_array::{RecordBatch, RecordBatchIterator};
use fastembed::TextEmbedding;
use lancedb::{Connection, arrow::arrow_schema::FieldRef};
use marrow::datatypes::{DataType, Field};
use serde::{Deserialize, Serialize};
use serde_arrow::schema::{SchemaLike, TracingOptions};
pub use vector_db_macro::VectorDbRecord;

#[derive(Clone)]
pub struct VectorDb {
    conn: Connection,
}

pub trait TryIntoRecordBatch: Iterator {
    fn try_into_record_batch(
        &mut self,
        model: &mut TextEmbedding,
    ) -> Result<RecordBatch, Box<dyn std::error::Error>>;
}

impl VectorDb {
    pub async fn try_new(path: &str) -> Result<Self, Box<dyn Error>> {
        let conn = lancedb::connect(path).execute().await?;
        Ok(Self { conn })
    }

    pub async fn create_table_with_data<'de, T>(
        &self,
        name: &str,
        data: &[T],
        embedding_fields: &[(&str, u16)],
    ) -> Result<(), Box<dyn Error>>
    where
        T: Deserialize<'de> + Serialize,
    {
        let batch = to_record_batch(data, embedding_fields)?;
        let schema = batch.schema();
        let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema.clone());
        self.conn.create_table(name, batches).execute().await?;
        Ok(())
    }
}

fn to_record_batch<'de, T>(
    data: &[T],
    embedding_fields: &[(&str, u16)],
) -> Result<RecordBatch, Box<dyn Error>>
where
    T: Deserialize<'de> + Serialize,
{
    let mut topts = TracingOptions::default();

    for (name, size) in embedding_fields {
        let field = fixed_size_list_field(*name, *size);
        topts = topts
            .overwrite(*name, Arc::new(field))
            .expect("embedding field created");
    }

    let fields = Vec::<FieldRef>::from_type::<T>(topts)?;
    let batch = serde_arrow::to_record_batch(&fields, &data)?;
    Ok(batch)
}

pub fn fixed_size_list_field(name: impl Into<String>, size: u16) -> Field {
    Field {
        name: name.into(),
        data_type: DataType::FixedSizeList(
            Box::new(Field {
                name: "item".into(),
                data_type: DataType::Float32,
                nullable: false,
                metadata: HashMap::new(),
            }),
            size as i32,
        ),
        nullable: false,
        metadata: HashMap::new(),
    }
}
