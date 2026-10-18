use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{bad_request, internal_server_error, see_other, unauthorized},
        page, route,
    },
    session,
    view::{View, component, view},
};

use crate::database::db;
use crate::models::User;

const INPUT_CLASS: &str = "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring";

#[derive(Debug, Deserialize)]
pub struct SettingsForm {
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub bio: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub confirm_password: String,
}

fn update_user_validation(mut user: User, form: &SettingsForm) -> std::result::Result<User, String> {
    let password = form.password.trim();
    let confirm_password = form.confirm_password.trim();

    if !password.is_empty() {
        if password != confirm_password {
            return Err("Error: New password and Confirm password do not match!".to_owned());
        }
        user = user.set_password(password.to_owned())?;
    }

    user.set_email(form.email.trim().to_owned())?
        .set_bio(form.bio.trim().to_owned())?
        .set_image(form.image.trim().to_owned())
}

async fn logout_session(cx: &Cx) -> Result<()> {
    if let Some(token_hash) = session::stop(cx).await? {
        sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
            .bind(token_hash.as_ref())
            .execute(db(cx))
            .await
            .map_err(topcoat::Error::from)?;
    }
    Ok(())
}

#[component]
async fn settings_editor(
    username: String,
    image: String,
    bio: String,
    email: String,
    error: Option<String>,
) -> Result<impl View> {
    Ok(view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60"
        >
            <title>"Settings"</title>
            <div
                class="block rounded-lg bg-white dark:bg-gray-800 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70"
            >
                <h5
                    class="mb-5 text-xl font-medium leading-tight text-neutral-800 dark:text-gray-200"
                >
                    "Update Your Settings."
                </h5>

                <form method="post" action="/settings">
                    if let Some(msg) = error {
                        <div class="mb-4 text-red-600 font-medium">(msg)</div>
                    }

                    <div class="mb-5">
                        <input
                            class=(INPUT_CLASS)
                            name="image"
                            type="text"
                            placeholder="URL of profile picture"
                            value=(image)
                        >
                    </div>

                    <div class="mb-5">
                        <input
                            class=(INPUT_CLASS)
                            name="username"
                            type="text"
                            disabled=""
                            placeholder="Username"
                            value=(username)
                        >
                    </div>

                    <div class="mb-5">
                        <textarea
                            name="bio"
                            class=(INPUT_CLASS)
                            rows="4"
                            placeholder="Short bio about you"
                        >
                            (bio)
                        </textarea>
                    </div>

                    <div class="mb-5">
                        <input
                            class=(INPUT_CLASS)
                            name="email"
                            type="email"
                            placeholder="Email (required)"
                            required=""
                            value=(email)
                        >
                    </div>

                    <div class="mb-5 relative">
                        <input
                            name="password"
                            type="password"
                            class=(INPUT_CLASS)
                            placeholder="New password"
                        >
                    </div>

                    <div class="mb-5 relative">
                        <input
                            name="confirm_password"
                            type="password"
                            class=(INPUT_CLASS)
                            placeholder="Confirm password"
                        >
                    </div>

                    <div class="flex justify-between mb-5">
                        <button
                            type="submit"
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-700"
                        >
                            "Update Settings"
                        </button>

                        <a
                            href="/"
                            class="bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg"
                        >
                            "Cancel"
                        </a>
                    </div>
                </form>
            </div>
        </div>
    })
}

#[page("/settings")]
pub async fn settings_page(cx: &Cx) -> Result<impl View> {
    let Some(preview) = crate::auth::current_user(cx).await? else {
        return Err(see_other("/login").into());
    };

    let user = User::get(&preview.username, cx)
        .await
        .map_err(topcoat::Error::from)?;

    let image = user.image.unwrap_or_default();
    let bio = user.bio.unwrap_or_default();

    Ok(view! {
        settings_editor(
            username: user.username,
            image: image,
            bio: bio,
            email: user.email,
            error: None,
        )
    })
}

#[route(POST "/settings")]
pub async fn settings_submit(
    cx: &Cx,
    Form(form): Form<SettingsForm>,
) -> Result<impl topcoat::router::response::IntoResponse> {
    let Some(preview) = crate::auth::current_user(cx).await? else {
        return Err(unauthorized().into());
    };

    let user = User::get(&preview.username, cx)
        .await
        .map_err(internal_server_error)?;

    let password_change = !form.password.trim().is_empty();
    let updated_user = update_user_validation(user, &form).map_err(bad_request)?;

    updated_user.update(cx).await.map_err(|e| {
        tracing::error!("Failed to update user: {e}");
        internal_server_error(e)
    })?;

    if password_change {
        if let Err(e) = logout_session(cx).await {
            tracing::error!("logout after password reset failed: {e}");
        }
        Ok(see_other("/login"))
    } else {
        Ok(see_other("/settings"))
    }
}
