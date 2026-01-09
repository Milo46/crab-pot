#[derive(Debug, Clone)]
pub struct SchemaNameVersion {
    pub name: String,
    pub version: Option<String>,
}

impl SchemaNameVersion {
    pub fn new(name: String, version: Option<String>) -> Self {
        Self { name, version }
    }

    pub fn with_version(name: String, version: String) -> Self {
        Self {
            name,
            version: Some(version),
        }
    }

    pub fn latest(name: String) -> Self {
        Self {
            name,
            version: None,
        }
    }
}
