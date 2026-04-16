use crate::router::RouteDecision;

pub trait Adapter: Send + Sync {
    fn chat_completion(&self, route: &RouteDecision, prompt: &str) -> String;
    fn embedding(&self, route: &RouteDecision, input: &str) -> Vec<f32>;
}

#[derive(Default)]
pub struct StubAdapter;

impl Adapter for StubAdapter {
    fn chat_completion(&self, route: &RouteDecision, prompt: &str) -> String {
        format!(
            "stubbed response via {} at {} ({} chars)",
            route.backend,
            route.endpoint,
            prompt.chars().count()
        )
    }

    fn embedding(&self, route: &RouteDecision, input: &str) -> Vec<f32> {
        let seed = (input.len() as f32).max(1.0);
        vec![
            seed,
            (route.profile.len() as f32) / 10.0,
            (route.backend.len() as f32) / 10.0,
        ]
    }
}

pub struct AdapterDispatcher {
    default_adapter: Box<dyn Adapter>,
}

impl Default for AdapterDispatcher {
    fn default() -> Self {
        Self {
            default_adapter: Box::<StubAdapter>::default(),
        }
    }
}

impl AdapterDispatcher {
    pub fn chat_completion(&self, route: &RouteDecision, prompt: &str) -> String {
        self.default_adapter.chat_completion(route, prompt)
    }

    pub fn embedding(&self, route: &RouteDecision, input: &str) -> Vec<f32> {
        self.default_adapter.embedding(route, input)
    }
}
