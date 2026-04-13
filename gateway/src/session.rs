#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
}

impl Session {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}
