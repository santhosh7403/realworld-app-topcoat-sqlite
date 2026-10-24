use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;
use topcoat::router::content::Form;
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    cookie::RouterBuilderCookieExt,
    router::{Router, RouterBuilderDiscoverExt, Slot, href, layout, page},
    runtime::{RouterBuilderRuntimeExt, signal},
    session::{RouterBuilderSessionExt, SessionConfig},
    view::{View, component, view},
};

mod database;
mod models;
use crate::{database::AppState, models::UserPreview};
mod auth;
mod components;
mod views;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://realworld-app-topcoat.db?mode=rwc".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let state = AppState { db: pool };

    let router = Router::builder()
        .cookies()
        .sessions(SessionConfig::default())
        .assets(AssetBundle::load().unwrap())
        .discover()
        .runtime()
        .app_context(state)
        .build();
    topcoat::start(router).await.unwrap();
    Ok(())
}

// ---------------------------------------------------------------------------
// Root layout — sticky top navbar + page content
// ---------------------------------------------------------------------------

#[layout("/")]
async fn root_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let current_user = crate::auth::current_user(cx).await?;
    let (theme_class, server_theme) = if let Some(u) = current_user.clone() {
        let dark = crate::models::User::get(&u.username, cx)
            .await
            .map(|x| x.theme_mode() == "dark")
            .unwrap_or(true);
        (
            if dark {
                "dark".to_string()
            } else {
                String::new()
            },
            true,
        )
    } else {
        ("dark".to_string(), false)
    };
    let user_in = signal(cx, || current_user);
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" class=(theme_class) data-server-theme=(if server_theme { "true" } else { "" })>
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>"My App — Topcoat"</title>
                topcoat::dev::script()
                topcoat::runtime::script()

                // Tailwind via CDN for quick styling
                <script src="https://cdn.tailwindcss.com"></script>
                <script>"tailwind.config = { darkMode: 'class' }"</script>

                // Font Awesome icons (same as the Dioxus original)
                <link
                    rel="stylesheet"
                    href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css"
                    integrity="sha512-DTOQO9RWCH3ppGqcWaEA1BIZOC6xxalwEsw9c2QQeAIftl+Vegovlnee1c9QX4TctnWMn13TZye+giMm8e2LwA=="
                    crossorigin="anonymous"
                    referrerpolicy="no-referrer"
                />

                // Minimal custom styles matching the original navitem look
                <style>
                    r#".navitem {
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        padding: 0.25rem 0.75rem;
                        border-radius: 0.5rem;
                        transition: background-color 0.15s, transform 0.15s;
                        cursor: pointer;
                        color: inherit;
                        text-decoration: none;
                    }
                    .navitem:hover {
                        background-color: rgba(255, 255, 255, 0.1);
                        transform: scale(1.05);
                    }
                    .navitem-icon {
                        font-size: 1.25rem;
                        margin-bottom: 0.15rem;
                    }
                    @media (min-width: 768px) {
                        .navitem-icon { font-size: 1.5rem; }
                    }
                    .theme-moon { display: none; }
                    .dark .theme-moon { display: block; }
                    .dark .theme-sun { display: none; }"#
                </style>

                <script>
                    r#"                    function toggleTheme() {
                        var html = document.documentElement;
                        var isDark = html.classList.toggle('dark');
                        try { localStorage.setItem('theme', isDark ? 'dark' : 'light'); } catch (e) {}
                        // Persist to DB for logged-in users
                        try {
                            fetch('/theme', {
                                method: 'POST',
                                credentials: 'same-origin',
                                headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                                body: 'theme=' + encodeURIComponent(isDark ? 'dark' : 'light')
                            });
                        } catch (e) {}
                    }
                    function togglePassword() {
                        var input = document.getElementById('password');
                        var icon = document.getElementById('password-eye');
                        if (!input || !icon) return;
                        if (input.type === 'password') {
                            input.type = 'text';
                            icon.classList.remove('fa-eye-slash');
                            icon.classList.add('fa-eye');
                        } else {
                            input.type = 'password';
                            icon.classList.remove('fa-eye');
                            icon.classList.add('fa-eye-slash');
                        }
                    }
                    (function() {
                        try {
                            if (document.documentElement.dataset.serverTheme === 'true') return;
                            var saved = localStorage.getItem('theme');
                            if (saved === 'light') {
                                document.documentElement.classList.remove('dark');
                            } else {
                                document.documentElement.classList.add('dark');
                            }
                        } catch (e) {}
                    })();"#
                </script>
            </head>
            <body
                class="bg-gray-100 dark:bg-gray-950 text-gray-900 dark:text-gray-100 min-h-screen flex flex-col"
            >
                navbar(user_in: &user_in)
                <main class="flex-1 w-full px-4 py-8">(slot)</main>
                footer()
            </body>
        </html>
    })
}

// ---------------------------------------------------------------------------
// Navbar component (mirrors the Dioxus NavBar structure)
// ---------------------------------------------------------------------------

#[component]
async fn navbar(user_in: &topcoat::runtime::Signal<Option<UserPreview>>) -> Result<impl View> {
    // let username = "demo_user";
    // let user_in = signal(cx, || None::<crate::models::UserPreview>);

    Ok(view! {
        <nav class="sticky top-0 z-10 shadow-md">
            <div
                class="bg-gray-800 text-white shadow-lg rounded-b-xl px-4 py-3 md:py-4"
            >
                <div class="flex justify-around items-center max-w-7xl mx-auto">
                    // Home
                    <a href="/" class="navitem group">
                        <i class="fas fa-home navitem-icon"></i>
                        <span class="text-xs md:text-base mt-1 font-semibold">
                            "Home"
                        </span>
                    </a>

                    if user_in.get().is_some() {
                        // New Article
                        <a href="/new-article" class="navitem group">
                            <i class="fa-solid fa-square-plus navitem-icon"></i>
                            <span class="text-xs md:text-base mt-1 font-semibold">
                                "New Article"
                            </span>
                        </a>

                        // Settings
                        <a href="/settings" class="navitem group">
                            <i class="fa-solid fa-gear navitem-icon"></i>
                            <span class="text-xs md:text-base mt-1 font-semibold">
                                "Settings"
                            </span>
                        </a>

                        // Profile
                        <a
                            href=(format!("/profile/{}", user_in.get().unwrap().username))
                            class="navitem group"
                        >
                            <i class="fa-regular fa-circle-user navitem-icon"></i>
                            <span class="text-xs md:text-base mt-1 font-semibold">
                                (user_in.get().unwrap().username) // (username)
                            </span>
                        </a>

                        <form
                            method="POST"
                            action=(href!(crate::views::logout::logout))
                        >
                            <button class="navitem group">
                                <i class="fa-solid fa-right-from-bracket navitem-icon"></i>
                                <span class="text-xs md:text-base mt-1 font-semibold">
                                    "Logout"
                                </span>
                            </button>
                        </form>
                    } else {
                        // Sign up
                        <a href="/signup" class="navitem group">
                            <i class="fa-solid fa-user-plus navitem-icon"></i>
                            <span class="text-xs md:text-base mt-1 font-semibold">
                                "Sign up"
                            </span>
                        </a>

                        // Login
                        <a href="/login" class="navitem group">
                            <i class="fa-solid fa-right-to-bracket navitem-icon"></i>
                            <span class="text-xs md:text-base mt-1 font-semibold">
                                "Login"
                            </span>
                        </a>
                    }

                    // Theme toggle (working)
                    <button
                        type="button"
                        onclick="toggleTheme()"
                        class="navitem group border-l border-gray-600 pl-3 ml-2"
                        title="Toggle theme"
                    >
                        <svg
                            class="theme-sun w-6 h-6 md:w-7 md:h-7 text-yellow-400"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z"
                            />
                        </svg>
                        <svg
                            class="theme-moon w-6 h-6 md:w-7 md:h-7 text-gray-400"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M21.752 15.002A9.718 9.718 0 0118 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 003 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 009.002-5.998z"
                            />
                        </svg>
                    </button>
                </div>
            </div>
        </nav>
    })
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

#[component]
async fn footer() -> Result<impl View> {
    Ok(view! {
        <footer
            class="bg-gray-200 text-gray-600 dark:bg-gray-900 dark:text-gray-400 text-center text-xs py-3 sticky bottom-0"
        >
            <a href="/" class="hover:underline">"MyApp"</a>
            <div>"© 2026 My-Website — Topcoat edition"</div>
        </footer>
    })
}

#[derive(Deserialize)]
struct PerPageForm {
    amount: i64,
}

#[derive(Deserialize)]
struct ThemeForm {
    theme: String,
}

#[page(POST "/items-per-page")]
async fn update_items_per_page(cx: &Cx, Form(form): Form<PerPageForm>) -> Result<impl View> {
    if let Some(user_preview) = crate::auth::current_user(cx).await? {
        if let Ok(mut user) = crate::models::User::get(&user_preview.username, cx).await {
            user = user.set_per_page_amount(form.amount.clamp(1, 100));
            let _ = user.update_per_page_amount(cx).await;
        }
    }
    Ok(view! { "" })
}

#[page(POST "/theme")]
async fn update_theme(cx: &Cx, Form(form): Form<ThemeForm>) -> Result<impl View> {
    if let Some(user_preview) = crate::auth::current_user(cx).await? {
        if let Ok(mut user) = crate::models::User::get(&user_preview.username, cx).await {
            user = user.set_theme_mode(form.theme);
            let _ = user.update_theme_mode(cx).await;
        }
    }
    Ok(view! { "" })
}

// Catch-all 404
#[page("/{*path}")]
async fn not_found() -> Result<impl View> {
    Ok(view! {
        <div class="max-w-2xl mx-auto text-center">
            <h1 class="text-3xl font-bold mb-4 text-red-500">"Page not found"</h1>
            <p class="text-gray-600 dark:text-gray-400 mb-6">
                "We are terribly sorry, but the page you requested doesn't exist."
            </p>
            <a
                href="/"
                class="inline-block px-4 py-2 bg-gray-800 text-white rounded-lg hover:bg-gray-700"
            >
                "Back to Home"
            </a>
        </div>
    })
}

// ---------------------------------------------------------------------------
// Reusable stub page component
// ---------------------------------------------------------------------------

#[component]
async fn stub_page(title: &str, description: &str) -> Result<impl View> {
    Ok(view! {
        <div class="max-w-2xl mx-auto">
            <h1 class="text-3xl font-bold mb-4">(title)</h1>
            <div class="bg-white dark:bg-gray-800 rounded-xl shadow p-6">
                <p class="text-gray-600 dark:text-gray-300">(description)</p>
                <p class="mt-4 text-sm text-gray-500">
                    "This is a placeholder. Replace with real content later."
                </p>
            </div>
        </div>
    })
}
