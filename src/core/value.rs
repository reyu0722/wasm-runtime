#[derive(Debug, Clone)]
pub struct Name(String);

impl Name {
    pub fn new(name: String) -> Self {
        Name(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
