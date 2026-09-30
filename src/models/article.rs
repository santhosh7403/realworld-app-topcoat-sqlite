use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPreview {
    pub username: String,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Article {
    pub slug: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub description: String,
    pub created_at: String,
    pub favorites_count: i64,
    pub comments_count: i64,
    pub tag_list: Vec<String>,
    pub author: UserPreview,
    pub fav: bool,
}
