use crate::Views::home::try_get_db;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
    let db = try_get_db(cx)
        .ok_or_else(|| topcoat::Error::msg(anyhow::anyhow!("db pool not available")))?;

    let Some(token_hash) = session::token_hash(cx).await? else {
        return Ok(None);
    };

    let row = sqlx::query_as::<_, (i64, String, i64)>(
        r#"
        SELECT u.id, u.username, s.expires_at
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.token_hash = ?
        "#,
    )
    .bind(token_hash.as_ref()) // TokenHash derefs to [u8; 32]
    .fetch_optional(db)
    .await
    .map_err(|e| topcoat::Error::from(e))?;

    let Some((id, username, expires_at)) = row else {
        return Ok(None);
    };

    // Reject expired sessions
    if unix_to_system_time(expires_at) <= SystemTime::now() {
        // Optionally delete the expired row here
        return Ok(None);
    }

    Ok(Some(crate::models::UserPreview { id, username }))
}

// fn pool(cx: &Cx) -> &SqlitePool {
//     app_context(cx)
// }

/// Convert SystemTime → unix seconds for storage
pub fn system_time_to_unix(t: SystemTime) -> i64 {
    t.duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64
}

/// Convert unix seconds → SystemTime
pub fn unix_to_system_time(secs: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(secs as u64)
}
