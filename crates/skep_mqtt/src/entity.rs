use skep_core::entity::SkipEntity;

#[derive(Debug)]
pub struct MqttEntity {}

impl SkipEntity for MqttEntity {
    fn has_entity_name(&self) -> bool {
        true
    }
}
