use crate::database::db;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use ref_cast::RefCast;
use serde::{Deserialize, Serialize};

use topcoat::context::Cx;

static EMAIL_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

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
    #[serde(default = "default_per_page")]
    pub per_page_amount: i64,
    #[serde(default = "default_theme")]
    pub theme_mode: String,
}

fn default_per_page() -> i64 {
    10
}

fn default_theme() -> String {
    "dark".to_string()
}

#[allow(unused)]
impl User {
    pub async fn get(username: &str, cx: &Cx) -> Result<Self, sqlx::Error> {
        sqlx::query!(
            r#"SELECT id, username, email, bio, image, password, 
            CAST(COALESCE(per_page_amount, 10) AS INTEGER) as "per_page_amount: i64", 
            COALESCE(theme_mode, 'dark') as "theme_mode: String" FROM users WHERE username = ? "#,
            username
        )
        .map(|x| User {
            id: x.id.expect("should give user id, else panic"),
            username: x.username,
            password: None,
            email: x.email,
            bio: x.bio,
            image: x.image,
            per_page_amount: x.per_page_amount.unwrap_or(10),
            theme_mode: x.theme_mode.unwrap_or("light".to_owned()),
        })
        .fetch_one(db(cx))
        .await
    }

    #[inline]
    pub fn username(&self) -> String {
        self.username.to_string()
    }
    #[inline]
    pub fn per_page_amount(&self) -> i64 {
        self.per_page_amount
    }
    #[inline]
    pub fn theme_mode(&self) -> String {
        self.theme_mode.clone()
    }
    #[inline]
    pub fn email(&self) -> String {
        self.email.to_string()
    }
    #[inline]
    pub fn bio(&self) -> Option<String> {
        self.bio.clone()
    }
    #[inline]
    pub fn image(&self) -> Option<String> {
        self.image.clone()
    }

    pub fn set_password(mut self, password: String) -> Result<Self, String> {
        if password.len() < 4 {
            return Err("You need to provide a stronger password".into());
        }
        self.password = Some(password);
        Ok(self)
    }

    pub fn set_username(mut self, username: String) -> Result<Self, String> {
        if username.len() < 4 {
            return Err(format!(
                "Username {username} is too short, at least 4 characters"
            ));
        }
        self.username = username;
        Ok(self)
    }

    fn validate_email(email: &str) -> bool {
        EMAIL_REGEX
            .get_or_init(|| regex::Regex::new(r"^[\w\-\.]+@([\w-]+\.)+\w{2,4}$").unwrap())
            .is_match(email)
    }

    pub fn set_email(mut self, email: String) -> Result<Self, String> {
        if !Self::validate_email(&email) {
            return Err(format!(
                "The email {email} is invalid, provide a correct one"
            ));
        }
        self.email = email;
        Ok(self)
    }

    pub fn set_bio(mut self, bio: String) -> Result<Self, String> {
        static BIO_MIN: usize = 10;
        if bio.is_empty() {
            self.bio = None;
        } else if bio.len() < BIO_MIN {
            return Err("bio too short, at least 10 characters".into());
        } else {
            self.bio = Some(bio);
        }
        Ok(self)
    }

    #[inline]
    pub fn set_image(mut self, image: String) -> Result<Self, String> {
        if image.is_empty() {
            self.image = None;
        } else if !image.starts_with("http") {
            return Err("Invalid image url!".into());
        } else {
            self.image = Some(image);
        }
        Ok(self)
    }

    #[inline]
    pub fn set_per_page_amount(mut self, amount: i64) -> Self {
        self.per_page_amount = amount;
        self
    }

    #[inline]
    pub fn set_theme_mode(mut self, theme: String) -> Self {
        self.theme_mode = theme;
        self
    }

    pub async fn get_user_frm_email(cx: &Cx, email: String) -> Result<Self, sqlx::Error> {
        sqlx::query!(
            r#"SELECT id, username, email, bio, image, password, 
            CAST(COALESCE(per_page_amount, 10) AS INTEGER) as "per_page_amount: i64", 
            COALESCE(theme_mode, 'dark') as "theme_mode: String" FROM users WHERE email=$1"#,
            email,
        )
        .map(|x| User {
            id: x.id.expect("should give id, else panic"),
            username: x.username,
            password: None,
            email: x.email,
            bio: x.bio,
            image: x.image,
            per_page_amount: x.per_page_amount.unwrap_or(10),
            theme_mode: x.theme_mode.unwrap_or("dark".to_string()),
        })
        .fetch_one(db(cx))
        .await
    }

    pub async fn insert(&self, cx: &Cx) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        // Hash the password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password =
            match argon2.hash_password(self.password.clone().unwrap().as_bytes(), &salt) {
                Ok(hash) => Some(hash.to_string()),
                Err(e) => {
                    tracing::error!("Failed to hash password: {:?}", e);
                    return Err(sqlx::Error::InvalidArgument(e.to_string()));
                }
            };
        let per_page_amount = default_per_page();
        let theme_mode = default_theme();

        sqlx::query!(
            "INSERT INTO Users(username, email, password, per_page_amount, theme_mode) VALUES ($1, $2, $3, $4, $5)",
            self.username,
            self.email,
            hashed_password,
            per_page_amount,
            theme_mode,
        )
        .execute(db(cx))
        .await
    }

    pub async fn update(&self, cx: &Cx) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let password_is_some = self.password.is_some();
        let mut hashed_password = None;
        if password_is_some {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            hashed_password =
                match argon2.hash_password(self.password.clone().unwrap().as_bytes(), &salt) {
                    Ok(hash) => Some(hash.to_string()),
                    Err(e) => {
                        tracing::error!("Failed to hash password: {:?}", e);
                        return Err(sqlx::Error::InvalidArgument(e.to_string()));
                    }
                };
        }
        sqlx::query!(
            "
UPDATE Users SET
    image=$2,
    bio=$3,
    email=$4,
    password=CASE WHEN $5 THEN $6 ELSE password END
WHERE username=$1",
            self.username,
            self.image,
            self.bio,
            self.email,
            password_is_some,
            hashed_password,
        )
        .execute(db(cx))
        .await
    }

    pub async fn update_per_page_amount(
        &self,
        cx: &Cx,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query!(
            "UPDATE Users SET per_page_amount=$2 WHERE username=$1",
            self.username,
            self.per_page_amount,
        )
        .execute(db(cx))
        .await
    }

    pub async fn update_theme_mode(
        &self,
        cx: &Cx,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query!(
            "UPDATE Users SET theme_mode=$2 WHERE username=$1",
            self.username,
            self.theme_mode,
        )
        .execute(db(cx))
        .await
    }
}

#[topcoat::runtime::record]
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct UserPreview {
    pub id: i64,
    pub username: String,
}

