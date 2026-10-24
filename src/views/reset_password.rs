use serde::Deserialize;
use topcoat::router::content::Form;
use topcoat::router::path_param;
use topcoat::router::request::headers;
use topcoat::{
    Result,
    context::Cx,
    router::{error::see_other, page},
    session,
    view::{View, view},
};

use crate::database::db;
use crate::models::User;

use uuid::Uuid;

path_param!(token);

const INPUT_CLASS: &str = "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring";

#[derive(Deserialize)]
pub struct ResetEmailForm {
    email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordForm {
    password: String,
    confirm: String,
}

/// Step 1 — GET /reset_password : show the email form.
#[page("/reset_password")]
pub async fn reset_password_page(cx: &Cx) -> Result<impl View> {
    Ok(reset_email_view(cx, None))
}

/// Step 1 — POST /reset_password : validate email, issue a reset token,
/// print the reset link for testing (dummy email) and step into step 2.
#[page(POST "/reset_password")]
pub async fn reset_password_request(
    cx: &Cx,
    Form(form): Form<ResetEmailForm>,
) -> Result<impl View> {
    // Delete expired tokens
    let _ = sqlx::query!("DELETE FROM password_reset_tokens WHERE expires_at < datetime('now')")
        .execute(db(cx))
        .await;

    let user = match User::get_user_frm_email(cx, form.email.clone()).await {
        Ok(u) => u,
        Err(_) => {
            return Ok(reset_email_view(
                cx,
                Some("Provided email not found.".into()),
            ));
        }
    };

    let token = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO password_reset_tokens(user_id, token, expires_at) VALUES ($1, $2, datetime('now', '+2 days'))",
        user.id,
        token
    )
    .execute(db(cx))
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert reset token: {}", e);
        topcoat::Error::msg("Failed to insert reset token")
    })?;

    let host = headers(cx)
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:8080");
    let scheme = if cfg!(debug_assertions) {
        "http"
    } else {
        "https"
    };
    let uri = format!("{scheme}://{host}/reset_password/{token}");
    tracing::info!("Reset URI: {uri}");
    // Dummy email: print the link that would go inside the email for testing.
    println!("RESET PASSWORD LINK (dummy email): {uri}");

    // Simulate the user clicking the link from the email → open step 2.
    Err(see_other(&format!("/reset_password/{token}")).into())
}

/// Step 2 — GET /reset_password/{token} : show the password form when the
/// token is valid and not expired, otherwise show an error.
#[page("/reset_password/{token}")]
pub async fn reset_password_set_page(cx: &Cx) -> Result<impl View> {
    let token: &str = path_param::<Token>(cx);

    let valid = sqlx::query!(
        "SELECT u.id FROM password_reset_tokens t JOIN users u ON u.id = t.user_id WHERE t.token = $1 AND t.expires_at > datetime('now')",
        token
    )
    .fetch_optional(db(cx))
    .await
    .map_err(|e| topcoat::Error::from(e))?;

    let Some(_) = valid else {
        return Ok(reset_password_view(
            cx,
            token,
            None,
            Some("Invalid or expired token.".into()),
        ));
    };

    Ok(reset_password_view(cx, token, None, None))
}

/// Step 2 — POST /reset_password/{token} : validate the token, check the
/// passwords match, set the new password, delete the token, log out and
/// redirect to login.
#[page(POST "/reset_password/{token}")]
pub async fn reset_password_set(cx: &Cx, Form(form): Form<ResetPasswordForm>) -> Result<impl View> {
    let token: &str = path_param::<Token>(cx);

    if form.password != form.confirm {
        return Ok(reset_password_view(
            cx,
            token,
            None,
            Some("Passwords do not match.".into()),
        ));
    }

    let record = sqlx::query!(
        "SELECT u.id, u.email FROM password_reset_tokens t JOIN users u ON u.id = t.user_id WHERE t.token = $1 AND t.expires_at > datetime('now')",
        token
    )
    .fetch_optional(db(cx))
    .await
    .map_err(|e| topcoat::Error::from(e))?;

    let Some(rec) = record else {
        return Ok(reset_password_view(
            cx,
            token,
            None,
            Some("Invalid or expired token.".into()),
        ));
    };

    let email = rec.email.clone();
    let mut user = User::get_user_frm_email(cx, email.clone())
        .await
        .map_err(|e| topcoat::Error::from(e))?;

    user = user
        .set_password(form.password)
        .map_err(|e| topcoat::Error::msg(e))?;

    if let Err(e) = user.update(cx).await {
        tracing::error!(email = email, ?e, "error while resetting the password");
        return Ok(reset_password_view(
            cx,
            token,
            None,
            Some("Failed to reset password.".into()),
        ));
    }

    // Delete the utilized token
    let _ = sqlx::query!("DELETE FROM password_reset_tokens WHERE token = $1", token)
        .execute(db(cx))
        .await;

    // Log out the user (clear session) and redirect to login.
    if let Some(token_hash) = session::stop(cx).await? {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(token_hash.as_ref())
            .execute(db(cx))
            .await
            .map_err(|e| topcoat::Error::from(e))?;
    }

    Err(see_other("/login").into())
}

/// Shared markup for the step 1 email form.
fn reset_email_view(cx: &Cx, error: Option<String>) -> impl View {
    let __cx = cx;
    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60"
        >
            <div
                class="block rounded-lg bg-white dark:bg-gray-800 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70"
            >
                <h5
                    class="mb-5 text-xl font-medium leading-tight text-neutral-800 dark:text-gray-200"
                >
                    "Reset Password"
                </h5>

                <form method="post" action="/reset_password">
                    <div class="mb-5">
                        <label
                            class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-2"
                            for="email"
                        >
                            "Provide your linked email address with your user account."
                        </label>
                        <input
                            class=(INPUT_CLASS)
                            id="email"
                            name="email"
                            type="text"
                            placeholder="Registered email"
                            required=""
                        >
                    </div>

                    <div class="mb-5 flex justify-between">
                        <button
                            type="submit"
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg disabled:bg-gray-400 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-blue-700"
                        >
                            "Send reset link"
                        </button>
                        <a
                            href="/login"
                            class="bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg"
                        >
                            "Cancel"
                        </a>
                    </div>

                    if let Some(msg) = error {
                        <p class="mb-5 text-red-500 font-medium">(msg)</p>
                    }
                </form>
            </div>
        </div>
    }
}

/// Shared markup for the step 2 password form.
fn reset_password_view(
    cx: &Cx,
    token: &str,
    success: Option<String>,
    error: Option<String>,
) -> impl View {
    let __cx = cx;
    let action = format!("/reset_password/{token}");

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60"
        >
            <div
                class="block rounded-lg bg-white dark:bg-gray-800 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70"
            >
                <h5
                    class="mb-5 text-xl font-medium leading-tight text-neutral-800 dark:text-gray-200"
                >
                    "Set a new password."
                </h5>

                <form method="post" action=(action)>
                    <div class="mb-5">
                        <label
                            class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-2"
                            for="password"
                        >
                            "New Password"
                        </label>
                        <input
                            class=(INPUT_CLASS)
                            id="password"
                            name="password"
                            type="password"
                            placeholder="New Password"
                            required=""
                        >
                    </div>

                    <div class="mb-5">
                        <label
                            class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-2"
                            for="confirm_password"
                        >
                            "Confirm Password"
                        </label>
                        <input
                            class=(INPUT_CLASS)
                            id="confirm_password"
                            name="confirm_password"
                            type="password"
                            placeholder="Confirm Password"
                            required=""
                        >
                    </div>

                    <div class="mb-5 flex justify-between">
                        <button
                            type="submit"
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg disabled:bg-gray-400 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-blue-700"
                        >
                            "Reset Password & Logout"
                        </button>
                        <a
                            href="/login"
                            class="bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg"
                        >
                            "Cancel"
                        </a>
                    </div>

                    if let Some(msg) = success {
                        <p class="mb-5 text-green-500 font-medium">(msg)</p>
                    }
                    if let Some(msg) = error {
                        <p class="mb-5 text-red-500 font-medium">(msg)</p>
                    }
                </form>
            </div>
        </div>
    }
}
