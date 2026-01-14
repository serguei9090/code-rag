use std::sync::Arc;
use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, Float32Array, Int32Array, Int64Array, FixedSizeListArray};

pub fn dummy_setup_fn() {
    println!("This function exists just to be indexed!");
    // Rust function
}