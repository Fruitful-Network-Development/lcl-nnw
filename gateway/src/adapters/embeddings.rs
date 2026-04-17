use crate::adapters::BackendAdapter;
use crate::router::ModelReference;

pub struct EmbeddingsAdapter;

impl BackendAdapter for EmbeddingsAdapter {
    fn key(&self) -> &'static str {
        "embeddings"
    }

    fn health(&self, endpoint: &str) -> Result<String, String> {
        Ok(format!("embeddings adapter healthy at {endpoint}"))
    }

    fn infer(
        &self,
        _endpoint: &str,
        model: &ModelReference,
        _prompt: &str,
    ) -> Result<String, String> {
        Err(format!(
            "embeddings adapter stub cannot run infer for model {}",
            model.alias
        ))
    }

    fn embeddings(
        &self,
        endpoint: &str,
        model: &ModelReference,
        input: &str,
    ) -> Option<Result<Vec<f32>, String>> {
        let _ = (endpoint, model);
        Some(Ok(vec![input.len() as f32]))
    }
}
