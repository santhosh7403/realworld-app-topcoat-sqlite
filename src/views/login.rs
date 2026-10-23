use crate::database::db;
use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{SeeOther, see_other},
        href, page, route,
    },
    runtime::{Event, signal},
    session,
    view::{View, view},
};

use crate::models::UserPreview;
// use sqlx::SqlitePool;

// use crate::auth;

/// Shared app state (already established in your app).
// pub struct AppState {
//     pub db: SqlitePool,
// }

// fn db(cx: &Cx) -> &SqlitePool {
//     &app_context::<AppState>(cx).db
// }

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

/// GET /login — render the form.
#[page("/login")]
pub async fn login_page(cx: &Cx) -> Result<impl View> {
    Ok(login_view(cx, None))
}

/// POST /login — validate credentials, start session, redirect or re-show errors.
#[page(POST "/login")]
async fn login_submit(cx: &Cx, Form(form): Form<LoginForm>) -> Result<impl View> {
    match verify_credentials(cx, &form.username, &form.password).await {
        Ok(user_id) => {
            // Start a fresh session token (prevents session fixation).
            let session = session::start(cx).await?;
            persist_session(cx, user_id, &session).await?;
            // 303 See Other → home
            Err(see_other("/").into())
        }
        Err(_) => {
            // Re-render the form with an error message.
            Ok(login_view(
                cx,
                Some("Login failed! : username or password incorrect".into()),
            ))
        }
    }
}

/// Shared markup for both GET and failed POST.
fn login_view(cx: &Cx, error: Option<String>) -> impl View {
    let __cx = cx;

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60"
        >
            <div
                class="block rounded-lg bg-white dark:bg-gray-800 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70"
            >
                <h5
                    class="mb-5 text-xl font-medium leading-tight text-gray-800 dark:text-gray-200"
                >
                    " Please login with your credentials."
                </h5>

                <form class="mb-10" id="login" method="post" action="/login">
                    <label
                        class="block text-gray-700 dark:text-gray-300 text-sm font-bold"
                        for="username"
                    >
                        "User Name"
                    </label>

                    <div class="mb-5">
                        <input
                            class="shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                            id="username"
                            name="username"
                            type="text"
                            placeholder="user name"
                            required=""
                        >
                    </div>

                    <label
                        class="block text-gray-700 dark:text-gray-300 text-sm font-bold"
                        for="password"
                    >
                        "Password"
                    </label>

                    <div class="mb-5 relative">
                        <input
                            class="shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                            id="password"
                            name="password"
                            type="password"
                            placeholder="password"
                            required=""
                        >
                        <span
                            class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500 dark:text-gray-400"
                            onclick="togglePassword()"
                        >
                            <i id="password-eye" class="far fa-eye-slash"></i>
                        </span>
                    </div>

                    <div class="flex flex-row-reverse space-x-4 space-x-reverse">
                        <button
                            type="submit"
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-2 text-white rounded-lg"
                        >
                            "Signin"
                        </button>

                        <a
                            href="/"
                            class="bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-2 text-white rounded-lg inline-block text-center"
                        >
                            "Cancel"
                        </a>

                        <a
                            class="p-2 text-gray-700 dark:text-gray-300 hover:text-blue-500 hover:underline"
                            href="/reset_password"
                        >
                            "Forgot password?"
                        </a>
                    </div>

                    if let Some(msg) = error {
                        <div>
                            <p class="block text-red-700 text-sm font-bold">(msg)</p>
                        </div>
                    }
                </form>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Auth helpers — adapt these to your real schema / password hashing
// ---------------------------------------------------------------------------

async fn verify_credentials(cx: &Cx, username: &str, password: &str) -> Result<i64, ()> {
    // let db = topcoat::context::try_app_context::<AppState>(cx)
    //     .map(|s| &s.db)
    //     .unwrap();
    // let db = try_get_db(cx).ok_or(())?;

    // let row: Option<(i64, String)> =
    //     sqlx::query_as("SELECT id, password FROM users WHERE username = ?")
    //         .bind(username)
    //         .fetch_optional(db(cx))
    //         .await
    //         .map_err(|_| ())?;
    let row: Option<(i64, String)> =
        sqlx::query_as::<_, (i64, String)>("SELECT id, password FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(db(cx)) // use the actual pool variable
            .await
            .map_err(|_| ())?;

    let Some((id, hash)) = row else {
        return Err(());
    };

    // Placeholder — use a real verifier:
    // if argon2::verify_encoded(&hash, password.as_bytes()).is_ok() { Ok(id) } else { Err(()) }
    if crate::auth::verify_password(password, &hash) {
        // if hash == password {
        // NEVER compare plaintext in production
        Ok(id)
    } else {
        Err(())
    }
}

async fn persist_session(cx: &Cx, user_id: i64, session: &session::Session) -> Result<()> {
    let token_hash = hex::encode(&*session.token_hash);
    // let expires_at: chrono::DateTime<chrono::Utc> = session.expires_at.into();
    let expires_at = session
        .expires_at
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // let db = try_get_db(cx)
    // .ok_or_else(|| topcoat::Error::msg(anyhow::anyhow!("db pool not available")))?;
    // let db = try_get_db(cx).ok_or_else(|| anyhow::anyhow!("db pool not available"))?;

    sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(token_hash)
        .bind(user_id)
        .bind(expires_at)
        .execute(db(cx))
        .await?;
    Ok(())
}
