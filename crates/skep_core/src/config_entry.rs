pub struct ConfigEntry {
    pub entry_id: String,
}

impl ConfigEntry {
    pub fn new(entity_id: Option<String>) -> Self {
        let entity_id = entity_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        Self {
            entry_id: entity_id,
        }
    }
}
