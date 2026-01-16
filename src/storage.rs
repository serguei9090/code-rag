use anyhow::Result;
use arrow_array::builder::{ListBuilder, StringBuilder};
use arrow_array::{
    FixedSizeListArray, Float32Array, Int32Array, Int64Array, RecordBatch, RecordBatchIterator,
    StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use futures_util::stream::TryStreamExt;
use lancedb::connect;
use lancedb::connection::Connection;
use lancedb::index::scalar::BTreeIndexBuilder;
use lancedb::query::{ExecutableQuery, QueryBase};
use std::sync::Arc;
// use lancedb::index::IndexType;

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

    pub async fn init(&self, dim: usize) -> Result<()> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("filename", DataType::Utf8, false),
            Field::new("code", DataType::Utf8, false),
            Field::new("line_start", DataType::Int32, false),
            Field::new("line_end", DataType::Int32, false),
            Field::new("last_modified", DataType::Int64, false),
            Field::new(
                "calls",
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
                true,
            ),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dim as i32,
                ),
                false,
            ),
        ]));

        if self
            .conn
            .open_table(&self.table_name)
            .execute()
            .await
            .is_err()
        {
            self.conn
                .create_empty_table(&self.table_name, schema)
                .execute()
                .await?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_chunks(
        &self,
        ids: Vec<String>,
        filenames: Vec<String>,
        code: Vec<String>,
        line_starts: Vec<i32>,
        line_ends: Vec<i32>,
        last_modified: Vec<i64>,
        calls: Vec<Vec<String>>,
        vectors: Vec<Vec<f32>>,
    ) -> Result<()> {
        let table = self.conn.open_table(&self.table_name).execute().await?;
        let table_schema = table.schema().await?;
        let vector_field = table_schema.field_with_name("vector").unwrap();
        let dim_val = if let DataType::FixedSizeList(_, d) = vector_field.data_type() {
            *d
        } else {
            768
        };

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("filename", DataType::Utf8, false),
            Field::new("code", DataType::Utf8, false),
            Field::new("line_start", DataType::Int32, false),
            Field::new("line_end", DataType::Int32, false),
            Field::new("last_modified", DataType::Int64, false),
            Field::new(
                "calls",
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
                true,
            ),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dim_val,
                ),
                false,
            ),
        ]));

        let id_array = StringArray::from(ids);
        let filename_array = StringArray::from(filenames);
        let code_array = StringArray::from(code);
        let line_starts_array = Int32Array::from(line_starts);
        let line_ends_array = Int32Array::from(line_ends);
        let last_modified_array = Int64Array::from(last_modified);

        // Build ListArray for calls
        let mut builder = ListBuilder::new(StringBuilder::new());
        for call_list in calls {
            for call in call_list {
                builder.values().append_value(call);
            }
            builder.append(true);
        }
        let calls_array = builder.finish();

        // Flatten vectors
        let flat_vectors: Vec<f32> = vectors.into_iter().flatten().collect();
        let values = Float32Array::from(flat_vectors);
        let field = Arc::new(Field::new("item", DataType::Float32, true));
        let vector_array = FixedSizeListArray::try_new(field, dim_val, Arc::new(values), None)?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(filename_array),
                Arc::new(code_array),
                Arc::new(line_starts_array),
                Arc::new(line_ends_array),
                Arc::new(last_modified_array),
                Arc::new(calls_array),
                Arc::new(vector_array),
            ],
        )?;

        let table = self.conn.open_table(&self.table_name).execute().await?;
        let reader = Box::new(RecordBatchIterator::new(vec![Ok(batch)], schema));
        table.add(reader).execute().await?;

        Ok(())
    }

    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filter: Option<String>,
    ) -> Result<Vec<RecordBatch>> {
        let table = self.conn.open_table(&self.table_name).execute().await?;
        let mut query = table.query().nearest_to(query_vector)?;

        if let Some(f) = filter {
            query = query.only_if(f);
        }

        let results = query
            .limit(limit)
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;
        Ok(results)
    }

    pub async fn get_indexed_metadata(&self) -> Result<std::collections::HashMap<String, i64>> {
        if self
            .conn
            .open_table(&self.table_name)
            .execute()
            .await
            .is_err()
        {
            return Ok(std::collections::HashMap::new());
        }

        let table = self.conn.open_table(&self.table_name).execute().await?;
        let mut stream = table
            .query()
            .select(lancedb::query::Select::Columns(vec![
                "filename".to_string(),
                "last_modified".to_string(),
            ]))
            .execute()
            .await?;

        let mut metadata = std::collections::HashMap::new();

        while let Some(batch) = stream.try_next().await? {
            let filenames: &StringArray = batch
                .column_by_name("filename")
                .ok_or(lancedb::Error::Runtime {
                    message: "Missing filename".into(),
                })?
                .as_any()
                .downcast_ref()
                .unwrap();

            if let Some(col) = batch.column_by_name("last_modified") {
                let mtimes: &Int64Array = col.as_any().downcast_ref().unwrap();
                for i in 0..batch.num_rows() {
                    let fname = filenames.value(i).to_string();
                    let mtime = mtimes.value(i);
                    metadata.insert(fname, mtime);
                }
            }
        }
        Ok(metadata)
    }

    pub async fn delete_file_chunks(&self, filename: &str) -> Result<()> {
        if self
            .conn
            .open_table(&self.table_name)
            .execute()
            .await
            .is_ok()
        {
            let table = self.conn.open_table(&self.table_name).execute().await?;
            let safe_filename = filename.replace("'", "''");
            table
                .delete(&format!("filename = '{}'", safe_filename))
                .await?;
        }
        Ok(())
    }
    pub async fn create_filename_index(&self) -> Result<()> {
        if self
            .conn
            .open_table(&self.table_name)
            .execute()
            .await
            .is_ok()
        {
            let table = self.conn.open_table(&self.table_name).execute().await?;
            let _ = table
                .create_index(
                    &["filename"],
                    lancedb::index::Index::BTree(BTreeIndexBuilder::default()),
                )
                .execute()
                .await;
        }
        Ok(())
    }
}
