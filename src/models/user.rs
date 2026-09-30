use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct User {
    #[serde(default)]
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    pub email: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    // #[serde(default = "default_per_page")]
    pub per_page_amount: i64,
    // #[serde(default = "default_theme")]
    pub theme_mode: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct UserPreview {
    pub id: i64,
    pub username: String,
}
