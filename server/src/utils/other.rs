use uuid::Uuid;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Watcher {
    pub uuid: Uuid,
    pub username: String,
    pub user_id: String,
    pub avatar_id: String,
}
