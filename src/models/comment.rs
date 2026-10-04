use crate::database::db;
use topcoat::context::Cx;

#[derive(Clone, Debug)]
pub struct Comment {
    pub id: i64,
    pub article: String,
    pub username: String,
    pub body: String,
    pub created_at: String,
    pub user_image: Option<String>,
}

pub async fn fetch_comments(cx: &Cx, slug: &str) -> Result<Vec<Comment>, sqlx::Error> {
    sqlx::query!(
        "
    SELECT c.id, c.article, c.body, c.created_at, u.username, u.image FROM Comments as c
        JOIN Users as u ON u.id=c.user_id
    WHERE c.article=$1
    ORDER BY c.created_at",
        slug
    )
    .map(|x| Comment {
        id: x.id,
        article: x.article,
        username: x.username,
        body: x.body,
        created_at: x.created_at.format(crate::models::DATE_FORMAT).to_string(),
        user_image: x.image,
    })
    .fetch_all(db(cx))
    .await
}

pub async fn insert_comment(
    cx: &Cx,
    article: &str,
    user_id: i64,
    body: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO Comments(article, user_id, body) VALUES ($1, $2, $3)",
        article,
        user_id,
        body,
    )
    .execute(db(cx))
    .await?;
    Ok(())
}

/// Deletes comment if owned by `user`. Returns article slug for redirect.
pub async fn delete_comment(cx: &Cx, id: i64, user_id: i64) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT article FROM Comments WHERE id = $1 AND user_id = $2",
        id,
        user_id,
    )
    .fetch_optional(db(cx))
    .await?;

    let Some(r) = row else {
        return Ok(None);
    };
    let slug = r.article;

    sqlx::query!(
        "DELETE FROM Comments WHERE id = $1 AND user_id = $2",
        id,
        user_id,
    )
    .execute(db(cx))
    .await?;

    Ok(Some(slug))
}
