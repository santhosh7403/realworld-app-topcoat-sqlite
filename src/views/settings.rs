use topcoat::{
    Result,
    context::Cx,
    router::{error::see_other, page},
    runtime::{Event, procedure, record, signal},
    session,
    view::{View, view},
};

use crate::database::db;

const INPUT_CLASS: &str = "shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring";

/// Procedure / UI result (enums are not runtime-safe).
#[record]
#[derive(Clone)]
pub struct SettingsResponse {
    /// "success" | "success_logout" | "passwords_not_match" | "validation_error" | "update_error" | "auth_error"
    pub kind: String,
    pub message: String,
}

/// Client form state for the settings page.
#[record]
#[derive(Clone)]
pub struct SettingsForm {
    pub username: String,
    pub orig_image: String,
    pub orig_bio: String,
    pub orig_email: String,
    pub image: String,
    pub bio: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub passwd_visible: bool,
    pub status: String,
    pub is_passwd_change: bool,
    pub button_label: String,
    pub button_disabled: bool,
    pub cancel_label: String,
}

impl SettingsForm {
    fn from_user(user: &crate::models::User) -> Self {
        Self {
            username: user.username.clone(),
            orig_image: user.image.clone().unwrap_or_default(),
            orig_bio: user.bio.clone().unwrap_or_default(),
            orig_email: user.email.clone(),
            image: user.image.clone().unwrap_or_default(),
            bio: user.bio.clone().unwrap_or_default(),
            email: user.email.clone(),
            password: String::new(),
            confirm_password: String::new(),
            passwd_visible: false,
            status: String::new(),
            is_passwd_change: false,
            button_label: "Update Settings".to_owned(),
            button_disabled: true,
            cancel_label: "Cancel".to_owned(),
        }
    }
}

fn update_user_validation(
    mut user: crate::models::User,
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: &str,
) -> std::result::Result<crate::models::User, SettingsResponse> {
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

    let user = match crate::models::User::get(&preview.username, cx).await {
        Ok(u) => u,
        Err(e) => {
            return Ok(SettingsResponse {
                kind: "update_error".to_owned(),
                message: format!("Could not load user: {e}"),
            });
        }
    };

    let username = user.username.clone();
    let password_change = !password.is_empty();

    let user = match update_user_validation(user, image, bio, email, password, &confirm_password) {
        Ok(u) => u,
        Err(resp) => return Ok(resp),
    };

    match user.update(cx).await {
        Ok(_) => {
            if password_change {
                if let Err(e) = logout_session(cx).await {
                    tracing::error!("logout after password reset failed: {e}");
                }
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
            tracing::error!("Problem while updating user: {username} with error {e}");
            Ok(SettingsResponse {
                kind: "update_error".to_owned(),
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

    let user = crate::models::User::get(&preview.username, cx)
        .await
        .map_err(topcoat::Error::from)?;

    let form = signal(cx, || SettingsForm::from_user(&user));

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
                            :value=$(form.get().image)
                            @input=$(|e: Event| {
                                let f = form.get();
                                let image = e.target.value;
                                let fields_unchanged = if image == f.orig_image {
                                    if f.bio == f.orig_bio {
                                        f.email == f.orig_email
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };
                                let pw_empty = if f.password.is_empty() {
                                    f.confirm_password.is_empty()
                                } else {
                                    false
                                };
                                form.set(
                                    SettingsForm {
                                        username: f.username,
                                        orig_image: f.orig_image,
                                        orig_bio: f.orig_bio,
                                        orig_email: f.orig_email,
                                        image: image,
                                        bio: f.bio,
                                        email: f.email,
                                        password: f.password,
                                        confirm_password: f.confirm_password,
                                        passwd_visible: f.passwd_visible,
                                        status: f.status,
                                        is_passwd_change: f.is_passwd_change,
                                        button_label: f.button_label,
                                        button_disabled: if fields_unchanged {
                                            pw_empty
                                        } else {
                                            false
                                        },
                                        cancel_label: f.cancel_label,
                                    },
                                );
                            })
                        >
                    </div>

                    <div class="mb-5">
                        <input
                            class=(INPUT_CLASS)
                            name="username"
                            type="text"
                            disabled=""
                            :placeholder=$(form.get().username)
                            :value=$(form.get().username)
                        >
                    </div>

                    <div class="mb-5">
                        <textarea
                            name="bio"
                            class=(INPUT_CLASS)
                            rows="4"
                            placeholder="Short bio about you"
                            :value=$(form.get().bio)
                            @input=$(|e: Event| {
                                let f = form.get();
                                let bio = e.target.value;
                                let fields_unchanged = if f.image == f.orig_image {
                                    if bio == f.orig_bio {
                                        f.email == f.orig_email
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };
                                let pw_empty = if f.password.is_empty() {
                                    f.confirm_password.is_empty()
                                } else {
                                    false
                                };
                                form.set(
                                    SettingsForm {
                                        username: f.username,
                                        orig_image: f.orig_image,
                                        orig_bio: f.orig_bio,
                                        orig_email: f.orig_email,
                                        image: f.image,
                                        bio: bio,
                                        email: f.email,
                                        password: f.password,
                                        confirm_password: f.confirm_password,
                                        passwd_visible: f.passwd_visible,
                                        status: f.status,
                                        is_passwd_change: f.is_passwd_change,
                                        button_label: f.button_label,
                                        button_disabled: if fields_unchanged {
                                            pw_empty
                                        } else {
                                            false
                                        },
                                        cancel_label: f.cancel_label,
                                    },
                                );
                            })
                        ></textarea>
                    </div>

                    <div class="mb-5">
                        <input
                            class=(INPUT_CLASS)
                            name="email"
                            type="text"
                            placeholder="Email (required)"
                            required=""
                            :value=$(form.get().email)
                            @input=$(|e: Event| {
                                let f = form.get();
                                let email = e.target.value;
                                let fields_unchanged = if f.image == f.orig_image {
                                    if f.bio == f.orig_bio {
                                        email == f.orig_email
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };
                                let pw_empty = if f.password.is_empty() {
                                    f.confirm_password.is_empty()
                                } else {
                                    false
                                };
                                form.set(
                                    SettingsForm {
                                        username: f.username,
                                        orig_image: f.orig_image,
                                        orig_bio: f.orig_bio,
                                        orig_email: f.orig_email,
                                        image: f.image,
                                        bio: f.bio,
                                        email: email,
                                        password: f.password,
                                        confirm_password: f.confirm_password,
                                        passwd_visible: f.passwd_visible,
                                        status: f.status,
                                        is_passwd_change: f.is_passwd_change,
                                        button_label: f.button_label,
                                        button_disabled: if fields_unchanged {
                                            pw_empty
                                        } else {
                                            false
                                        },
                                        cancel_label: f.cancel_label,
                                    },
                                );
                            })
                        >
                    </div>

                    <div class="mb-5 relative">
                        <input
                            name="password"
                            class=(INPUT_CLASS)
                            placeholder="New password"
                            :type=$(if form.get().passwd_visible {
                                "text"
                            } else {
                                "password"
                            })
                            :value=$(form.get().password)
                            @input=$(|e: Event| {
                                let f = form.get();
                                let password = e.target.value;
                                let confirm_password = f.confirm_password;
                                let fields_unchanged = if f.image == f.orig_image {
                                    if f.bio == f.orig_bio {
                                        f.email == f.orig_email
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };
                                let pw_empty = if password.is_empty() {
                                    confirm_password.is_empty()
                                } else {
                                    false
                                };
                                let both_filled = if password.is_empty() {
                                    false
                                } else {
                                    !confirm_password.is_empty()
                                };
                                let matched = if both_filled {
                                    password == confirm_password
                                } else {
                                    false
                                };

                                let status = if pw_empty {
                                    "".to_owned()
                                } else if matched {
                                    "Passwords matched!".to_owned()
                                } else if both_filled {
                                    "Passords do not match!".to_owned()
                                } else {
                                    f.status
                                };

                                let button_label = if matched {
                                    "Reset Password & Logout".to_owned()
                                } else {
                                    "Update Settings".to_owned()
                                };

                                form.set(
                                    SettingsForm {
                                        username: f.username,
                                        orig_image: f.orig_image,
                                        orig_bio: f.orig_bio,
                                        orig_email: f.orig_email,
                                        image: f.image,
                                        bio: f.bio,
                                        email: f.email,
                                        password: password,
                                        confirm_password: confirm_password,
                                        passwd_visible: f.passwd_visible,
                                        status: status,
                                        is_passwd_change: matched,
                                        button_label: button_label,
                                        button_disabled: if fields_unchanged {
                                            pw_empty
                                        } else {
                                            false
                                        },
                                        cancel_label: f.cancel_label,
                                    },
                                );
                            })
                        >
                        <span
                            class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500 dark:text-gray-400"
                            @click=$(|_e: Event| {
                                let f = form.get();
                                form.set(
                                    SettingsForm {
                                        username: f.username,
                                        orig_image: f.orig_image,
                                        orig_bio: f.orig_bio,
                                        orig_email: f.orig_email,
                                        image: f.image,
                                        bio: f.bio,
                                        email: f.email,
                                        password: f.password,
                                        confirm_password: f.confirm_password,
                                        passwd_visible: !f.passwd_visible,
                                        status: f.status,
                                        is_passwd_change: f.is_passwd_change,
                                        button_label: f.button_label,
                                        button_disabled: f.button_disabled,
                                        cancel_label: f.cancel_label,
                                    },
                                );
                            })
                        >
                            <i
                                :class=$(if form.get().passwd_visible {
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
                            class=(INPUT_CLASS)
                            placeholder="Confirm password"
                            :type=$(if form.get().passwd_visible {
                                "text"
                            } else {
                                "password"
                            })
                            :value=$(form.get().confirm_password)
                            @input=$(|e: Event| {
                                let f = form.get();
                                let password = f.password;
                                let confirm_password = e.target.value;
                                let fields_unchanged = if f.image == f.orig_image {
                                    if f.bio == f.orig_bio {
                                        f.email == f.orig_email
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };
                                let pw_empty = if password.is_empty() {
                                    confirm_password.is_empty()
                                } else {
                                    false
                                };
                                let both_filled = if password.is_empty() {
                                    false
                                } else {
                                    !confirm_password.is_empty()
                                };
                                let matched = if both_filled {
                                    password == confirm_password
                                } else {
                                    false
                                };

                                let status = if pw_empty {
                                    "".to_owned()
                                } else if matched {
                                    "Passwords matched!".to_owned()
                                } else if both_filled {
                                    "Passords do not match!".to_owned()
                                } else {
                                    f.status
                                };

                                let button_label = if matched {
                                    "Reset Password & Logout".to_owned()
                                } else {
                                    "Update Settings".to_owned()
                                };

                                form.set(
                                    SettingsForm {
                                        username: f.username,
                                        orig_image: f.orig_image,
                                        orig_bio: f.orig_bio,
                                        orig_email: f.orig_email,
                                        image: f.image,
                                        bio: f.bio,
                                        email: f.email,
                                        password: password,
                                        confirm_password: confirm_password,
                                        passwd_visible: f.passwd_visible,
                                        status: status,
                                        is_passwd_change: matched,
                                        button_label: button_label,
                                        button_disabled: if fields_unchanged {
                                            pw_empty
                                        } else {
                                            false
                                        },
                                        cancel_label: f.cancel_label,
                                    },
                                );
                            })
                        >
                        <span
                            class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500"
                            @click=$(|_e: Event| {
                                let f = form.get();
                                form.set(
                                    SettingsForm {
                                        username: f.username,
                                        orig_image: f.orig_image,
                                        orig_bio: f.orig_bio,
                                        orig_email: f.orig_email,
                                        image: f.image,
                                        bio: f.bio,
                                        email: f.email,
                                        password: f.password,
                                        confirm_password: f.confirm_password,
                                        passwd_visible: !f.passwd_visible,
                                        status: f.status,
                                        is_passwd_change: f.is_passwd_change,
                                        button_label: f.button_label,
                                        button_disabled: f.button_disabled,
                                        cancel_label: f.cancel_label,
                                    },
                                );
                            })
                        >
                            <i
                                :class=$(if form.get().passwd_visible {
                                    "far fa-eye"
                                } else {
                                    "far fa-eye-slash"
                                })
                            ></i>
                        </span>
                    </div>

                    <div class="mb-5">
                        <p
                            :class=$(if form.get().status.starts_with("Successful.") {
                                "font-medium text-green-500"
                            } else if form.get().status.starts_with("Passwords matched") {
                                "font-medium text-green-500"
                            } else {
                                "font-medium text-red-500"
                            })
                        >
                            $(form.get().status)
                        </p>
                    </div>

                    <div class="flex justify-between mb-5">
                        <button
                            type="button"
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg disabled:bg-gray-400 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-blue-700"
                            :disabled=$(form.get().button_disabled)
                            @click=$(async |_e: Event| {
                                let f = form.get();
                                let resp = settings_update(
                                    f.image.to_owned(),
                                    f.bio.to_owned(),
                                    f.email.to_owned(),
                                    f.password.to_owned(),
                                    f.confirm_password.to_owned(),
                                ).await;

                                if resp.kind == "success_logout" {
                                    form.set(
                                        SettingsForm {
                                            username: f.username,
                                            orig_image: f.orig_image,
                                            orig_bio: f.orig_bio,
                                            orig_email: f.orig_email,
                                            image: f.image,
                                            bio: f.bio,
                                            email: f.email,
                                            password: f.password,
                                            confirm_password: f.confirm_password,
                                            passwd_visible: f.passwd_visible,
                                            status: resp.message,
                                            is_passwd_change: f.is_passwd_change,
                                            button_label: f.button_label,
                                            button_disabled: true,
                                            cancel_label: "Back to Home".to_owned(),
                                        },
                                    );
                                    raw!("window.location.href = '/login'");
                                } else if resp.kind == "success" {
                                    form.set(
                                        SettingsForm {
                                            username: f.username,
                                            orig_image: f.image.to_owned(),
                                            orig_bio: f.bio.to_owned(),
                                            orig_email: f.email.to_owned(),
                                            image: f.image,
                                            bio: f.bio,
                                            email: f.email,
                                            password: "".to_owned(),
                                            confirm_password: "".to_owned(),
                                            passwd_visible: f.passwd_visible,
                                            status: resp.message,
                                            is_passwd_change: false,
                                            button_label: "Update Settings".to_owned(),
                                            button_disabled: true,
                                            cancel_label: "Back to Home".to_owned(),
                                        },
                                    );
                                } else if resp.kind == "auth_error" {
                                    raw!("window.location.href = '/login'");
                                } else {
                                    form.set(
                                        SettingsForm {
                                            username: f.username,
                                            orig_image: f.orig_image,
                                            orig_bio: f.orig_bio,
                                            orig_email: f.orig_email,
                                            image: f.image,
                                            bio: f.bio,
                                            email: f.email,
                                            password: f.password,
                                            confirm_password: f.confirm_password,
                                            passwd_visible: f.passwd_visible,
                                            status: resp.message,
                                            is_passwd_change: f.is_passwd_change,
                                            button_label: f.button_label,
                                            button_disabled: f.button_disabled,
                                            cancel_label: f.cancel_label,
                                        },
                                    );
                                }
                            })
                        >
                            $(form.get().button_label)
                        </button>

                        <button
                            type="button"
                            :class=$(if form.get().status.starts_with("Successful.") {
                                "bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg"
                            } else {
                                "bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg"
                            })
                            @click=$(async |_e: Event| {
                                if form.get().status.starts_with("Successful.") {
                                    raw!("window.location.href = '/'")
                                } else {
                                    raw!(
                                        r#"
                                        if (window.history.length > 1) {
                                            window.history.back();
                                        } else {
                                            window.location.href = '/';
                                        }
                                    "#
                                    )
                                }
                            })
                        >
                            $(form.get().cancel_label)
                        </button>
                    </div>
                </form>
            </div>
        </div>
    })
}
