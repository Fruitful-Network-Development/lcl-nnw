use std::collections::HashMap;

use crate::router::ModelReference;

pub mod embeddings;
pub mod llama_cpp;

pub trait BackendAdapter: Send + Sync {
    fn key(&self) -> &'static str;
    fn health(&self, endpoint: &str) -> Result<String, String>;
    fn infer(&self, endpoint: &str, model: &ModelReference, prompt: &str)
        -> Result<String, String>;

    fn embeddings(
        &self,
        _endpoint: &str,
        _model: &ModelReference,
        _input: &str,
    ) -> Option<Result<Vec<f32>, String>> {
        None
    }
}

pub struct AdapterRegistry {
    adapters: HashMap<String, Box<dyn BackendAdapter>>,
}

impl AdapterRegistry {
    pub fn default_with_builtin() -> Self {
        let mut registry = Self {
            adapters: HashMap::new(),
        };
        registry.register(Box::new(llama_cpp::LlamaCppAdapter));
        registry.register(Box::new(embeddings::EmbeddingsAdapter));
        registry
    }

    pub fn register(&mut self, adapter: Box<dyn BackendAdapter>) {
        self.adapters.insert(adapter.key().to_string(), adapter);
    }

    pub fn get(&self, key: &str) -> Option<&dyn BackendAdapter> {
        self.adapters.get(key).map(Box::as_ref)
    }
}
