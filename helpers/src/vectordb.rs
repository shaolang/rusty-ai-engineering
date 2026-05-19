extern crate self as helpers;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

pub use fastembed::{self, TextEmbedding};
pub use tokio_rusqlite::{self, Connection, Row, Transaction};
pub use zerocopy;

use crate::Result;
pub use vectordb_macro::Embed;

pub trait Embed: Send + Sync + for<'a, 'b> From<&'a Row<'b>> {
    fn create_sqlite_table_stmt() -> String;
    fn create_vector_index_stmt() -> String;
    fn search_stmt() -> String;
    fn insert(self, tx: &Transaction, embedder: &mut TextEmbedding) -> Result<()>;
}

pub struct VectorDb {
    conn: Connection,
    embedder: Arc<Mutex<TextEmbedding>>,
}

impl VectorDb {
    pub async fn try_connect(db_path: impl AsRef<Path>) -> Result<Self> {
        let embedder = Arc::new(Mutex::new(load_embedder()?));
        let conn = connect_sqlite(db_path.as_ref()).await?;
        Ok(Self { conn, embedder })
    }

    pub async fn create_table<E: Embed>(&self) -> Result<()> {
        let batch_stmts = [
            "BEGIN",
            &E::create_sqlite_table_stmt(),
            &E::create_vector_index_stmt(),
            "END",
        ]
        .join(";");
        self.conn
            .call(move |conn| conn.execute_batch(&batch_stmts))
            .await?;
        Ok(())
    }

    pub async fn insert<E: Embed + 'static>(&self, es: Vec<E>) -> Result<()> {
        let embedder = Arc::clone(&self.embedder);

        self.conn
            .call(move |conn| {
                let mut embedder = embedder.lock().unwrap();
                let tx = conn.transaction()?;
                for e in es {
                    e.insert(&tx, &mut embedder).expect("record inserted");
                }
                tx.commit()
            })
            .await?;
        Ok(())
    }

    pub async fn search<E: Embed + 'static>(
        &self,
        query: impl AsRef<str>,
        topk: usize,
    ) -> Result<Vec<E>> {
        let embedding: Vec<f32> = {
            let mut embedder = self.embedder.lock().unwrap();
            let es = embedder.embed(&[query.as_ref()], None)?;
            es.first().unwrap().to_owned()
        };

        let results = self
            .conn
            .call(move |conn| {
                use zerocopy::IntoBytes;
                let mut stmt = conn.prepare(&E::search_stmt()).unwrap();

                let rows = stmt.query_map(
                    tokio_rusqlite::rusqlite::params![embedding.as_bytes(), topk],
                    |row| Ok(row.into()),
                )?;

                rows.collect()
            })
            .await?;

        Ok(results)
    }
}

fn load_embedder() -> Result<TextEmbedding> {
    let options =
        fastembed::InitOptions::new(fastembed::EmbeddingModel::BGESmallENV15).with_max_length(384);

    Ok(TextEmbedding::try_new(options)?)
}

async fn connect_sqlite(path: impl AsRef<Path>) -> Result<Connection> {
    unsafe {
        use tokio_rusqlite::rusqlite::ffi::{
            sqlite3, sqlite3_api_routines, sqlite3_auto_extension,
        };
        type SqliteExtFn = unsafe extern "C" fn(
            *mut sqlite3,
            *mut *mut std::ffi::c_char,
            *const sqlite3_api_routines,
        ) -> std::ffi::c_int;

        let f: SqliteExtFn = std::mem::transmute(sqlite_vec::sqlite3_vec_init as *const ());
        sqlite3_auto_extension(Some(f));
    };
    Ok(Connection::open(path).await?)
}
