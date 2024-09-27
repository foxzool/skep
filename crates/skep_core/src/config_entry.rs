use nanoid::nanoid;

pub struct ConfigEntry {
    pub entry_id: String,
}

impl ConfigEntry {
    pub fn new(entity_id: Option<String>) -> Self {
        let entity_id = entity_id.unwrap_or_else(|| nanoid!());
        Self {
            entry_id: entity_id,
        }
    }
}
