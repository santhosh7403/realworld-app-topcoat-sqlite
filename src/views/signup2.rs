use topcoat::{
    Result,
    context::Cx,
    router::{href, page},
    runtime::{Event, link, procedure, record, signal},
    view::{View, view},
};

// Procedure result (also a record — enums still not runtime-safe)
#[record]
#[derive(Clone)]
pub struct SignupResponse {
    pub kind: String, // "success" | "validation_error" | "create_error"
    pub message: String,
}

// All client form UI state in ONE signal
#[record]
#[derive(Clone)]
pub struct SignupForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub passwd_visible: bool,
    pub status: String,
    pub button_label: String,
}

impl SignupForm {
    fn new() -> Self {
        Self {
            username: String::new(),
            email: String::new(),
            password: String::new(),
            passwd_visible: false,
            status: String::new(),
            button_label: "Create Account".to_owned(),
        }
    }
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
    // Single signal — this is the 0.10-friendly shape
    let form = signal(cx, SignupForm::new);

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
                            :value=$(form.get().username)
                            @input=$(|e: Event| {
                                let f = form.get();
                                form.set(
                                    SignupForm {
                                        username: e.target.value,
                                        email: f.email,
                                        password: f.password,
                                        passwd_visible: f.passwd_visible,
                                        status: f.status,
                                        button_label: f.button_label,
                                    },
                                );
                            })
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
                            :value=$(form.get().email)
                            @input=$(|e: Event| {
                                let f = form.get();
                                form.set(
                                    SignupForm {
                                        username: f.username,
                                        email: e.target.value,
                                        password: f.password,
                                        passwd_visible: f.passwd_visible,
                                        status: f.status,
                                        button_label: f.button_label,
                                    },
                                );
                            })
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
                            :type=$(if form.get().passwd_visible {
                                "text"
                            } else {
                                "password"
                            })
                            placeholder="Password"
                            required=""
                            :value=$(form.get().password)
                            @input=$(|e: Event| {
                                let f = form.get();
                                form.set(
                                    SignupForm {
                                        username: f.username,
                                        email: f.email,
                                        password: e.target.value,
                                        passwd_visible: f.passwd_visible,
                                        status: f.status,
                                        button_label: f.button_label,
                                    },
                                );
                            })
                        >
                        <span
                            class="absolute inset-y-0 right-0 flex items-center pr-3 cursor-pointer text-gray-500 dark:text-gray-400"
                            @click=$(|_e: Event| {
                                let f = form.get();
                                form.set(
                                    SignupForm {
                                        username: f.username,
                                        email: f.email,
                                        password: f.password,
                                        passwd_visible: !f.passwd_visible,
                                        status: f.status,
                                        button_label: f.button_label,
                                    },
                                );
                            })
                        >
                            $(if form.get().passwd_visible {
                                "👁"
                            } else {
                                "👁‍🗨"
                            })
                        </span>
                    </div>

                    <div class="flex flex-row-reverse space-x-4 space-x-reverse">
                        <button
                            type="button"
                            class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg"
                            @click=$(async |_e: Event| {
                                let f = form.get();

                                if f.status.starts_with("Signup Successful") {
                                    raw!("window.location.href = '/login'")
                                } else {
                                    // Await yields the Ok value; Err fails the browser expression.
                                    let resp = signup_action(
                                        f.username.to_owned(),
                                        f.email.to_owned(),
                                        f.password.to_owned(),
                                    ).await;

                                    if resp.kind == "success" {
                                        form.set(
                                            SignupForm {
                                                username: f.username,
                                                email: f.email,
                                                password: f.password,
                                                passwd_visible: f.passwd_visible,
                                                status: resp.message,
                                                button_label: "Login to Account Created?".to_owned(),
                                            },
                                        );
                                    } else if resp.kind == "validation_error" {
                                        form.set(
                                            SignupForm {
                                                username: f.username,
                                                email: f.email,
                                                password: f.password,
                                                passwd_visible: f.passwd_visible,
                                                status: resp.message,
                                                button_label: f.button_label,
                                            },
                                        );
                                    } else if resp.kind == "create_error" {
                                        form.set(
                                            SignupForm {
                                                username: f.username,
                                                email: f.email,
                                                password: f.password,
                                                passwd_visible: f.passwd_visible,
                                                status: resp.message,
                                                button_label: f.button_label,
                                            },
                                        );
                                    } else {
                                        form.set(
                                            SignupForm {
                                                username: f.username,
                                                email: f.email,
                                                password: f.password,
                                                passwd_visible: f.passwd_visible,
                                                status: "There was some problem with signup, try again later".to_owned(

                                                ),
                                                button_label: f.button_label,
                                            },
                                        );
                                    }
                                }
                            })
                        >
                            $(form.get().button_label)
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
                            :class=$(if form
                                .get()
                                .status
                                .starts_with("Signup Successful") {
                                "font-medium text-green-500"
                            } else {
                                "font-medium text-red-500"
                            })
                        >
                            $(form.get().status)
                        </span>
                    </div>
                </form>
            </div>
        </div>
    })
}
