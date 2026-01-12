use std::sync::Arc;
use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, Float32Array, Int32Array, FixedSizeListArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::connection::Connection;
use lancedb::{connect, Result};
use lancedb::query::{ExecutableQuery, QueryBase};
use futures_util::stream::TryStreamExt;
use std::error::Error;

pub struct Storage {
    conn: Connection,
    table_name: String,
}

impl Storage {
    pub async fn new(uri: &str) -> Result<Self> {
        let conn = connect(uri).execute().await?;
        Ok(Self {
            conn,
            table_name: "code_chunks".to_string(),
        })
    }

    pub async fn init(&self) -> Result<()> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("filename", DataType::Utf8, false),
            Field::new("code", DataType::Utf8, false),
            Field::new("line_start", DataType::Int32, false),
            Field::new("line_end", DataType::Int32, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    768,
                ),
                false,
            ),
        ]));

        if self.conn.open_table(&self.table_name).execute().await.is_err() {
            self.conn
                .create_empty_table(&self.table_name, schema)
                .execute()
                .await?;
        }
        Ok(())
    }

    pub async fn add_chunks(
        &self,
        ids: Vec<String>,
        filenames: Vec<String>,
        code: Vec<String>,
        line_starts: Vec<i32>,
        line_ends: Vec<i32>,
        vectors: Vec<Vec<f32>>,
    ) -> std::result::Result<(), Box<dyn Error>> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("filename", DataType::Utf8, false),
            Field::new("code", DataType::Utf8, false),
            Field::new("line_start", DataType::Int32, false),
            Field::new("line_end", DataType::Int32, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    768,
                ),
                false,
            ),
        ]));

        let id_array = StringArray::from(ids);
        let filename_array = StringArray::from(filenames);
        let code_array = StringArray::from(code);
        let line_starts_array = Int32Array::from(line_starts);
        let line_ends_array = Int32Array::from(line_ends);
        
        // Flatten vectors
        let flat_vectors: Vec<f32> = vectors.into_iter().flatten().collect();
        let values = Float32Array::from(flat_vectors);
        let field = Arc::new(Field::new("item", DataType::Float32, true));
        let vector_array = FixedSizeListArray::try_new(
            field,
            768,
            Arc::new(values),
            None,
        )?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(filename_array),
                Arc::new(code_array),
                Arc::new(line_starts_array),
                Arc::new(line_ends_array),
                Arc::new(vector_array),
            ],
        )?;

        let table = self.conn.open_table(&self.table_name).execute().await?;
        let reader = Box::new(RecordBatchIterator::new(vec![Ok(batch)], schema));
        table.add(reader).execute().await?;

        Ok(())
    }

    pub async fn search(&self, query_vector: Vec<f32>, limit: usize) -> Result<Vec<RecordBatch>> {
        let table = self.conn.open_table(&self.table_name).execute().await?;
        let results = table
            .query()
            .nearest_to(query_vector)?
            .limit(limit)
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;
        Ok(results)
    }
}

