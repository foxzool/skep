use bevy_reflect::Reflect;

#[derive(Debug, Reflect, Default)]
pub struct Context {
    pub id: String,
    pub user_id: Option<String>,
    pub parent_id: Option<String>,
}
