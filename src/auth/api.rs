// use crate::views::home::try_get_db;
use crate::database::db;

use topcoat::{
    Result,
    context::Cx,
    session::{self},
};

// #[route(POST "/api/user")]
// pub async fn current_user() -> Result<Option<crate::models::User>> {
//     todo!()
// }

/// Look up the current session → user (or None)
pub async fn current_user(cx: &Cx) -> Result<Option<crate::models::UserPreview>> {
    // let db = try_get_db(cx)
    //     .ok_or_else(|| topcoat::Error::msg(anyhow::anyhow!("db pool not available")))?;

    let Some(token_hash) = session::token_hash(cx).await? else {
        return Ok(None);
    };
    let encoded_hash = hex::encode(*token_hash);
    // println!("At current_user - token_hash : {:?}", token_hash);

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

    // Reject expired sessions
    // if unix_to_system_time(expires_at) <= SystemTime::now() {
    //     // Optionally delete the expired row here
    //     return Ok(None);
    // }
}

// fn pool(cx: &Cx) -> &SqlitePool {
//     app_context(cx)
// }

// / Convert SystemTime → unix seconds for storage
// pub fn system_time_to_unix(t: SystemTime) -> i64 {
//     t.duration_since(UNIX_EPOCH)
//         .unwrap_or(Duration::ZERO)
//         .as_secs() as i64
// }

// / Convert unix seconds → SystemTime
// pub fn unix_to_system_time(secs: i64) -> SystemTime {
//     UNIX_EPOCH + Duration::from_secs(secs as u64)
// }

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
