use topcoat::{
    Result,
    context::Cx,
    router::{href, page},
    runtime::{Event, link, procedure, record, signal},
    view::{View, view},
};

#[record]
#[derive(Clone)]
pub struct SignupResponse {
    pub kind: String, // "success" | "validation_error" | "create_error"
    pub message: String,
}

#[procedure]
async fn signup_action(
    cx: &Cx,
    username: String,
    email: String,
    password: String,
) -> Result<SignupResponse> {
    if username.trim().is_empty() {
        return Ok(SignupResponse {
            kind: "validation_error".to_owned(),
            message: "Problem while validating: username is required.".to_owned(),
        });
    }
    if !email.contains('@') {
        return Ok(SignupResponse {
            kind: "validation_error".to_owned(),
            message: "Problem while validating: invalid email.".to_owned(),
        });
    }
    if password.len() < 8 {
        return Ok(SignupResponse {
            kind: "validation_error".to_owned(),
            message: "Problem while validating: password must be at least 8 characters.".to_owned(),
        });
    }

    let result = crate::auth::signup_action(cx, username, email, password).await;
    match result {
        Ok(_) => Ok(SignupResponse {
            kind: "success".to_owned(),
            message: "Signup Successful".to_owned(),
        }),
        Err(e) => Ok(SignupResponse {
            kind: "create_error".to_owned(),
            message: format!("Problem while creating user: {e}."),
        }),
    }
}

#[page("/signup")]
async fn signup(cx: &Cx) -> Result<impl View> {
    let username = signal(cx, || String::new());
    let email = signal(cx, || String::new());
    let password = signal(cx, || String::new());
    let passwd_visible = signal(cx, || false);
    let status = signal(cx, || String::new());
    let button_label = signal(cx, || "Create Account".to_owned());

    Ok(view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60"
        >
            <div
                class="block rounded-lg bg-white dark:bg-gray-800 w-2/5 p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70"
            >
                <h5
                    class="mb-5 text-xl font-medium leading-tight text-neutral-800 dark:text-gray-200"
                >
                    "Create an account."
                </h5>

                <title>"Account Creation"</title>

                <p class="text-xs-center py-6">
                    <span class="text-blue-500 font-medium">
                        link(
                            href: href!(crate::views::login::login_page),
                            "Have an account already? Click here to login "
                        )
                    </span>
                </p>

                <form>
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
                            :value=$(username.get())
                            @input=$(|e: Event| username.set(e.target.value))
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
                            :value=$(email.get())
                            @input=$(|e: Event| email.set(e.target.value))
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
                            :type=$(if passwd_visible.get() {
                                "text"
                            } else {
                                "password"
                            })
                            placeholder="Password"
                            required=""
                            :value=$(password.get())
                            @input=$(|e: Event| password.set(e.target.value))
                        >
                        <span
                            class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500 dark:text-gray-400"
                            @click=$(|_e: Event| passwd_visible.set(
                                    !passwd_visible.get(),
                                ))
                        >
                            $(if passwd_visible.get() { "👁" } else { "👁‍🗨" })
                        </span>
                    </div>

                    <div class="flex flex-row-reverse space-x-4 space-x-reverse">
                        <button
                            type="button"
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg"
                            @click=$(async |_e: Event| {
                                if status.get().starts_with("Signup Successful") {
                                    raw!("window.location.href = '/login'")
                                } else {
                                    let resp = signup_action(
                                        username.get().to_owned(),
                                        email.get().to_owned(),
                                        password.get().to_owned(),
                                    ).await;

                                    if resp.kind == "success" {
                                        status.set(resp.message);
                                        button_label.set("Login to Account Created?".to_owned());
                                    } else if resp.kind == "validation_error" {
                                        status.set(resp.message);
                                    } else if resp.kind == "create_error" {
                                        status.set(resp.message);
                                    } else {
                                        status.set(
                                            "There was some problem with signup, try again later".to_owned(

                                            ),
                                        );
                                    }
                                }
                            })
                        >
                            $(button_label.get())
                        </button>

                        <a
                            href="/"
                            class="bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg inline-block text-center"
                        >
                            "Cancel"
                        </a>
                    </div>

                    <div>
                        <span
                            :class=$(if status.get().starts_with("Signup Successful") {
                                "font-medium text-green-500"
                            } else {
                                "font-medium text-red-500"
                            })
                        >
                            $(status.get())
                        </span>
                    </div>
                </form>
            </div>
        </div>
    })
}
