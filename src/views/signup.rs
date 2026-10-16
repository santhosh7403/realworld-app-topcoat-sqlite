use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{content::Form, error::see_other, href, page},
    runtime::{Event, link, signal},
    view::{View, view},
};

use crate::database::db;

#[derive(Debug, Deserialize)]
struct SignupForm {
    username: String,
    email: String,
    password: String,
}

/// GET /signup — render the form.
#[page("/signup")]
pub async fn signup(cx: &Cx) -> Result<impl View> {
    Ok(signup_view(cx, None))
}

/// POST /signup — validate credentials, create user, redirect or re-show errors.
#[page(POST "/signup")]
async fn signup_submit(cx: &Cx, Form(form): Form<SignupForm>) -> Result<impl View> {
    // Validate
    if form.username.trim().is_empty() {
        return Ok(signup_view(cx, Some("Username is required".into())));
    }
    if !form.email.contains('@') {
        return Ok(signup_view(cx, Some("Invalid email".into())));
    }
    if form.password.len() < 8 {
        return Ok(signup_view(
            cx,
            Some("Password must be at least 8 characters".into()),
        ));
    }

    // Insert user
    let result = sqlx::query("INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)")
        .bind(&form.username)
        .bind(&form.email)
        .bind(&form.password) // hash this in real code
        .execute(db(cx))
        .await;

    match result {
        Ok(_) => {
            // Success → redirect to login page
            Err(see_other("/login").into())
        }
        Err(e) => Ok(signup_view(
            cx,
            Some(format!("Problem while creating user: {e}")),
        )),
    }
}

/// Shared markup for both GET and failed POST.
fn signup_view(cx: &Cx, error: Option<String>) -> impl View {
    let __cx = cx;
    let passwd_visible = signal(cx, || false);

    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60">
            <div class="block rounded-lg bg-white dark:bg-gray-800 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70">

                <h5 class="mb-5 text-xl font-medium leading-tight text-neutral-800 dark:text-gray-200">
                    "Create an account."
                </h5>

                <title>"Account Creation"</title>

                <p class="text-xs-center py-6">
                    <span class="text-blue-500 font-medium">
                        // 0.10 client-side navigation (no full reload)
                        link(href: href!(crate::views::login::login_page), "Have an account already? Click here to login ")
                    </span>
                </p>

                <form method="post" action="/signup">
                    <label
                        class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-2"
                        for="username"
                    >
                        "User Name"
                    </label>
                    <div class="mb-5">
                        <input
                            class="shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring"
                            id="username"
                            name="username"
                            type="text"
                            placeholder="username"
                            required=""
                        >
                    </div>

                    <label
                        class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-2"
                        for="email"
                    >
                        "Email"
                    </label>
                    <div class="mb-5">
                        <input
                            class="shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring"
                            id="email"
                            name="email"
                            type="email"
                            placeholder="Email"
                            required=""
                        >
                    </div>

                    <label
                        class="block text-gray-700 dark:text-gray-300 text-sm font-bold mb-2"
                        for="password"
                    >
                        "Password"
                    </label>
                    <div class="mb-5 relative">
                        <input
                            class="shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring"
                            id="password"
                            name="password"
                            :type=$(if passwd_visible.get() { "text" } else { "password" })
                            placeholder="Password"
                            required=""
                        >
                        <span
                            class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500 dark:text-gray-400"
                            @click=$(|_e: Event| passwd_visible.toggle())
                        >
                            $(if passwd_visible.get() { "👁" } else { "👁‍🗨" })
                        </span>
                    </div>

                    <div class="flex flex-row-reverse space-x-4 space-x-reverse">
                        <button
                            type="submit"
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg"
                        >
                            "Create Account"
                        </button>

                        <a
                            href="/"
                            class="bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg inline-block text-center"
                        >
                            "Cancel"
                        </a>
                    </div>

                    if let Some(msg) = error {
                        <div>
                            <span class="font-medium text-red-500">
                                (msg)
                            </span>
                        </div>
                    }
                </form>
            </div>
        </div>
    }
}
