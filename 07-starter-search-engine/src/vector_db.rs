use arrow_array::{RecordBatch, RecordBatchIterator};
use lancedb::Connection;

pub struct VectorDb {
    conn: Connection,
}

pub trait IntoRecordBatches {
    fn into(self) -> RecordBatch;
}

impl VectorDb {
    pub async fn connect(dir: &str) -> Self {
        let conn = lancedb::connect(dir)
            .execute()
            .await
            .expect("connected to lancedb");
        Self { conn }
    }

    pub async fn create_table(&self, name: &str, data: impl IntoRecordBatches) {
        let batches = data.into();
        let schema = batches.schema();
        let batch_iterator = RecordBatchIterator::new(vec![Ok(batches)], schema.clone());

        self.conn
            .drop_all_tables(&[])
            .await
            .expect("dropped all tables"); // for simplicity
        self.conn
            .create_table(name, Box::new(batch_iterator))
            .execute()
            .await
            .expect("created table in lancedb");
    }
}
