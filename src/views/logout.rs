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

// #[page(GET "/logout")]
// async fn logout_page(cx: &Cx) -> Result<impl View> {
//     Ok(logout(cx))
// }

#[route(POST "/logout")]
pub async fn logout(cx: &Cx) -> Result<SeeOther> {
    // let db = try_get_db(cx)
    //     .ok_or_else(|| topcoat::Error::msg(anyhow::anyhow!("db pool not available")))?;

    // stop() clears the cookie and returns the hash so we can delete the DB row
    if let Some(token_hash) = session::stop(cx).await? {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(token_hash.as_ref())
            .execute(db(cx))
            .await
            .map_err(|e| topcoat::Error::from(e))?;
    }

    Ok(see_other(href!(crate::views::home::home).resolve(cx)))
}
