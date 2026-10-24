use crate::database::db;

use topcoat::{
    Result,
    context::Cx,
    session::{self},
};

pub async fn current_user(cx: &Cx) -> Result<Option<crate::models::UserPreview>> {
    let Some(token_hash) = session::token_hash(cx).await? else {
        return Ok(None);
    };
    let encoded_hash = hex::encode(*token_hash);

    let row = sqlx::query_as::<_, (i64, String, i64)>(
        r#"
        SELECT u.id, u.username, s.expires_at
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.token_hash = ?
        "#,
    )
    // .bind(token_hash.as_ref()) // TokenHash derefs to [u8; 32]
    .bind(encoded_hash) // TokenHash derefs to [u8; 32]
    .fetch_optional(db(cx))
    .await
    .map_err(|e| topcoat::Error::from(e))?;

    if let Some((id, username, expires_at)) = row {
        let expires_at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(expires_at as u64);
        if expires_at < std::time::SystemTime::now() {
            return Ok(None);
        } else {
            // println!("Triggered current_user : {:#?}", username.clone());
            Ok(Some(crate::models::UserPreview { id, username }))
        }
    } else {
        Ok(None)
    }
}

#[derive(serde::Deserialize, Clone, serde::Serialize)]
pub enum SignupResponse {
    ValidationError(String),
    CreateUserError(String),
    Success,
}

pub fn validate_signup(
    username: String,
    email: String,
    password: String,
) -> Result<crate::models::User, String> {
    crate::models::User::default()
        .set_username(username)?
        .set_password(password)?
        .set_email(email)
}

pub async fn signup_action(
    cx: &Cx,
    username: String,
    email: String,
    password: String,
) -> Result<SignupResponse, String> {
    match validate_signup(username.clone(), email, password.clone()) {
        Ok(user) => match user.insert(cx).await {
            Ok(_) => Ok(SignupResponse::Success),
            Err(x) => {
                let x = x.to_string();
                Ok(if x.contains("UNIQUE constraint failed: Users.email") {
                    SignupResponse::CreateUserError("Duplicated email".to_string())
                } else if x.contains("UNIQUE constraint failed: Users.username") {
                    SignupResponse::CreateUserError("Duplicated user".to_string())
                } else {
                    tracing::error!("error from DB: {}", x);
                    SignupResponse::CreateUserError(
                        "There is some problem in user creation, check log".to_string(),
                    )
                })
            }
        },
        Err(x) => Ok(SignupResponse::ValidationError(x)),
    }
}
