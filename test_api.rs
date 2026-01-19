// Test to verify fastembed API signature
use fastembed::TextEmbedding;

fn test_embed_signature() {
    // This test checks if embed() takes &self or &mut self
    let model: TextEmbedding = todo!();

    // If this compiles with &self, fastembed IS thread-safe
    let _result = model.embed(vec!["test".to_string()], None);

    // If compiler says "cannot borrow as mutable", then fastembed requires &mut self
}
