use crate::database::db;
// use crate::views::home::try_get_db;

use topcoat::{
    Result,
    context::Cx,
    router::{
        error::{SeeOther, see_other},
        href, route,
    },
    session,
};

#[route(POST "/logout")]
pub async fn logout(cx: &Cx) -> Result<SeeOther> {
    if let Some(token_hash) = session::stop(cx).await? {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(token_hash.as_ref())
            .execute(db(cx))
            .await
            .map_err(|e| topcoat::Error::from(e))?;
    }

    Ok(see_other(href!(crate::views::home::home).resolve(cx)))
}
