use crate::adapters::BackendAdapter;
use crate::router::ModelReference;

pub struct LlamaCppAdapter;

impl BackendAdapter for LlamaCppAdapter {
    fn key(&self) -> &'static str {
        "llama_cpp"
    }

    fn health(&self, endpoint: &str) -> Result<String, String> {
        Ok(format!("llama_cpp adapter healthy at {endpoint}"))
    }

    fn infer(
        &self,
        endpoint: &str,
        model: &ModelReference,
        prompt: &str,
    ) -> Result<String, String> {
        Ok(format!(
            "llama_cpp inference dispatched: endpoint={endpoint} model_alias={} prompt_len={}",
            model.alias,
            prompt.len()
        ))
    }
}
