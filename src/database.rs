use sqlx::SqlitePool;
use topcoat::context::{Cx, app_context};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

pub fn db(cx: &Cx) -> &SqlitePool {
    &app_context::<crate::AppState>(cx).db
}
