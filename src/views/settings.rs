use topcoat::{
    Result,
    context::Cx,
    router::{error::see_other, page},
    runtime::{Event, procedure, record, signal},
    view::{View, view},
};

use crate::models::User;

const INPUT_CLASS: &str = "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring";

#[record]
#[derive(Clone)]
pub struct SettingsResponse {
    pub kind: String,
    pub message: String,
}

fn update_user_validation(
    mut user: User,
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: &str,
) -> std::result::Result<User, SettingsResponse> {
    if !password.is_empty() {
        if password != confirm_password {
            return Err(SettingsResponse {
                kind: "passwords_not_match".to_owned(),
                message: "Error: New password and Confirm password do not match!".to_owned(),
            });
        }
        user = user.set_password(password).map_err(|e| SettingsResponse {
            kind: "validation_error".to_owned(),
            message: e,
        })?;
    }

    user.set_email(email)
        .map_err(|e| SettingsResponse {
            kind: "validation_error".to_owned(),
            message: e,
        })?
        .set_bio(bio)
        .map_err(|e| SettingsResponse {
            kind: "validation_error".to_owned(),
            message: e,
        })?
        .set_image(image)
        .map_err(|e| SettingsResponse {
            kind: "validation_error".to_owned(),
            message: e,
        })
}

#[procedure]
async fn settings_update(
    cx: &Cx,
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: String,
) -> Result<SettingsResponse> {
    let Some(preview) = crate::auth::current_user(cx).await? else {
        return Ok(SettingsResponse {
            kind: "auth_error".to_owned(),
            message: "You need to be authenticated".to_owned(),
        });
    };

    let user = match User::get(&preview.username, cx).await {
        Ok(u) => u,
        Err(e) => {
            return Ok(SettingsResponse {
                kind: "server_error".to_owned(),
                message: format!("Could not load user: {e}"),
            });
        }
    };

    let password_change = !password.is_empty();

    let user = match update_user_validation(user, image, bio, email, password, &confirm_password) {
        Ok(u) => u,
        Err(resp) => return Ok(resp),
    };

    match user.update(cx).await {
        Ok(_) => {
            if password_change {
                Ok(SettingsResponse {
                    kind: "success_logout".to_owned(),
                    message: "Successful.".to_owned(),
                })
            } else {
                Ok(SettingsResponse {
                    kind: "success".to_owned(),
                    message: "Successful.".to_owned(),
                })
            }
        }
        Err(e) => {
            tracing::error!("Problem while updating user: {e}");
            Ok(SettingsResponse {
                kind: "server_error".to_owned(),
                message: "Problem while updating user".to_owned(),
            })
        }
    }
}

#[page("/settings")]
pub async fn settings_page(cx: &Cx) -> Result<impl View> {
    let Some(preview) = crate::auth::current_user(cx).await? else {
        return Err(see_other("/login").into());
    };

    let user = User::get(&preview.username, cx)
        .await
        .map_err(topcoat::Error::from)?;

    let username = user.username.clone();

    let image = signal(cx, || user.image.clone().unwrap_or_default());
    let bio = signal(cx, || user.bio.clone().unwrap_or_default());
    let email = signal(cx, || user.email.clone());
    let pw = signal(cx, || String::new());
    let confirm_pw = signal(cx, || String::new());
    let passwd_visible = signal(cx, || false);
    let update_status = signal(cx, || String::new());
    let button_label = signal(cx, || "Update Settings".to_owned());
    let cancel_label = signal(cx, || "Cancel".to_owned());

    let orig_image = signal(cx, || user.image.clone().unwrap_or_default());
    let orig_bio = signal(cx, || user.bio.clone().unwrap_or_default());
    let orig_email = signal(cx, || user.email.clone());

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

                <form>
                    <div class="mb-5">
                        <input
                            class=(INPUT_CLASS)
                            name="image"
                            type="text"
                            placeholder="URL of profile picture"
                            :value=$(image.get())
                            @input=$(|e: Event| image.set(e.target.value))
                        >
                    </div>

                    <div class="mb-5">
                        <input
                            class=(INPUT_CLASS)
                            name="username"
                            type="text"
                            disabled=""
                            placeholder=(username.clone())
                            value=(username.clone())
                        >
                    </div>

                    <div class="mb-5">
                        <textarea
                            name="bio"
                            class=(INPUT_CLASS)
                            rows="4"
                            placeholder="Short bio about you"
                            :value=$(bio.get())
                            @input=$(|e: Event| bio.set(e.target.value))
                        ></textarea>
                    </div>

                    <div class="mb-5">
                        <input
                            class=(INPUT_CLASS)
                            name="email"
                            type="text"
                            placeholder="Email (required)"
                            required=""
                            :value=$(email.get())
                            @input=$(|e: Event| email.set(e.target.value))
                        >
                    </div>

                    <div class="mb-5 relative">
                        <input
                            name="password"
                            autocomplete="new-password"
                            class=(INPUT_CLASS)
                            placeholder="New password"
                            :type=$(if passwd_visible.get() {
                                "text"
                            } else {
                                "password"
                            })
                            :value=$(pw.get())
                            @input=$(|e: Event| {
                                let p = e.target.value;
                                pw.set(p.clone());

                                let c = confirm_pw.get();
                                let pw_empty = if p.is_empty() {
                                    c.is_empty()
                                } else {
                                    false
                                };
                                let both_filled = if p.is_empty() {
                                    false
                                } else {
                                    !c.is_empty()
                                };
                                let matched = if both_filled { p == c } else { false };

                                let new_status = if pw_empty {
                                    "".to_owned()
                                } else if matched {
                                    "Passwords matched!".to_owned()
                                } else if both_filled {
                                    "Passords do not match!".to_owned()
                                } else {
                                    update_status.get()
                                };

                                let new_label = if matched {
                                    "Reset Password & Logout".to_owned()
                                } else {
                                    "Update Settings".to_owned()
                                };

                                update_status.set(new_status);
                                button_label.set(new_label);
                            })
                        >
                        <span
                            class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500 dark:text-gray-400"
                            @click=$(|_e: Event| passwd_visible.toggle())
                        >
                            <i
                                :class=$(if passwd_visible.get() {
                                    "far fa-eye"
                                } else {
                                    "far fa-eye-slash"
                                })
                            ></i>
                        </span>
                    </div>

                    <div class="mb-5 relative">
                        <input
                            name="confirm_password"
                            autocomplete="new-password"
                            class=(INPUT_CLASS)
                            placeholder="Confirm password"
                            :type=$(if passwd_visible.get() {
                                "text"
                            } else {
                                "password"
                            })
                            :value=$(confirm_pw.get())
                            @input=$(|e: Event| {
                                let c = e.target.value;
                                confirm_pw.set(c.clone());

                                let p = pw.get();
                                let pw_empty = if p.is_empty() {
                                    c.is_empty()
                                } else {
                                    false
                                };
                                let both_filled = if p.is_empty() {
                                    false
                                } else {
                                    !c.is_empty()
                                };
                                let matched = if both_filled { p == c } else { false };

                                let new_status = if pw_empty {
                                    "".to_owned()
                                } else if matched {
                                    "Passwords matched!".to_owned()
                                } else if both_filled {
                                    "Passords do not match!".to_owned()
                                } else {
                                    update_status.get()
                                };

                                let new_label = if matched {
                                    "Reset Password & Logout".to_owned()
                                } else {
                                    "Update Settings".to_owned()
                                };

                                update_status.set(new_status);
                                button_label.set(new_label);
                            })
                        >
                        <span
                            class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500"
                            @click=$(|_e: Event| passwd_visible.toggle())
                        >
                            <i
                                :class=$(if passwd_visible.get() {
                                    "far fa-eye"
                                } else {
                                    "far fa-eye-slash"
                                })
                            ></i>
                        </span>
                    </div>

                    <div class="mb-5">
                        <p
                            :class=$(if update_status.get().starts_with("Successful.") {
                                "font-medium text-green-500"
                            } else {
                                if update_status.get().starts_with("Passwords matched") {
                                    "font-medium text-green-500"
                                } else {
                                    "font-medium text-red-500"
                                }
                            })
                        >
                            $(update_status.get())
                        </p>
                    </div>

                    <div class="flex justify-between mb-5">
                        <button
                            type="button"
                            :disabled=$(
                                if image.get() != orig_image.get() {
                                    false
                                } else if bio.get() != orig_bio.get() {
                                    false
                                } else if email.get() != orig_email.get() {
                                    false
                                } else if pw.get().is_empty() {
                                    true
                                } else if pw.get() != confirm_pw.get() {
                                    true
                                } else if pw.get().len() < 4 {
                                    true
                                } else {
                                    false
                                }
                            )
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg disabled:bg-gray-400 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-blue-700"
                            @click=$(async |_e: Event| {
                                let resp = settings_update(
                                    image.get().to_owned(),
                                    bio.get().to_owned(),
                                    email.get().to_owned(),
                                    pw.get().to_owned(),
                                    confirm_pw.get().to_owned(),
                                ).await;

                                if resp.kind == "success_logout" {
                                    update_status.set(resp.message);
                                    raw!(
                                        "const f = document.createElement('form'); f.method = 'POST'; f.action = '/logout'; document.body.appendChild(f); f.submit();"
                                    );
                                } else if resp.kind == "success" {
                                    orig_image.set(image.get());
                                    orig_bio.set(bio.get());
                                    orig_email.set(email.get());

                                    pw.set("".to_owned());
                                    confirm_pw.set("".to_owned());

                                    update_status.set(resp.message);
                                    button_label.set("Update Settings".to_owned());
                                    cancel_label.set("Back to Home".to_owned());
                                } else if resp.kind == "auth_error" {
                                    raw!("window.location.href = '/login'");
                                } else {
                                    update_status.set(resp.message);
                                }
                            })
                        >
                            $(button_label.get())
                        </button>

                        <a
                            href="/"
                            :class=$(if update_status.get().starts_with("Successful.") {
                                "bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg"
                            } else {
                                "bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg"
                            })
                        >
                            $(cancel_label.get())
                        </a>
                    </div>
                </form>
            </div>
        </div>
    })
}
