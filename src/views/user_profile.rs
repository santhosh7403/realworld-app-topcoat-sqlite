use topcoat::{
    Result,
    context::Cx,
    router::{page, parse_query_params, path_param, query_params},
    view::{View, component, view},
};

use crate::components::{
    article_preview::article_preview_list, prev_next_button::prev_next_profile,
};
use crate::models::User;

#[query_params]
struct ProfileQuery {
    page: Option<i64>,
    amount: Option<i64>,
    favourites: Option<bool>,
}

impl ProfileQuery {
    fn page(&self) -> i64 {
        self.page.unwrap_or(0).max(0)
    }

    fn favourites(&self) -> bool {
        self.favourites.unwrap_or(false)
    }
}

// ---------- models ----------

#[derive(Clone)]
pub struct UserProfileModel {
    pub user: User,
}

// ---------- data access (no server functions needed) ----------

// async fn load_profile_articles(
//     cx: &Cx,
//     username: &str,
//     favourites: bool,
//     page: i64,
//     amount: i64,
//     current_user_id: Option<i64>,
// ) -> Result<Vec<Article>> {
//     // replace with your real query
//     Article::for_user_profile_home(username, favourites, page, amount, current_user_id)
//         .await
//         .map_err(|e| {
//             tracing::error!("profile articles: {e:?}");
//             topcoat::Error::msg("Could not retrieve articles")
//         })
// }

async fn load_user_profile(cx: &Cx, username: &str) -> Result<UserProfileModel> {
    let user = crate::models::User::get(username, cx).await.map_err(|e| {
        tracing::error!("user_profile: {e:?}");
        topcoat::Error::msg("Could not retrieve user")
    })?;

    Ok(UserProfileModel { user })
}

// ---------- page ----------

path_param!(username);

#[page("/profile/{username}")]
async fn profile(cx: &Cx) -> Result<impl View> {
    // You can also read query params for pagination here if you want
    let username = path_param::<Username>(cx);
    println!("At page profile : {}", username);
    Ok(view! {
        <div class="bg-gray-200 dark:bg-gray-900">
            <div
                class="mx-auto sm:px-6 lg:px-8 bg-gray-200 dark:bg-gray-900 px-2 py-2 sm:px-0 text-gray-800 dark:text-gray-200"
            >
                profile_page(username: username)
            </div>
        </div>
    })
}

// ---------- components ----------

#[component]
async fn profile_page(cx: &Cx, username: &str) -> Result<impl View> {
    // For demo we hard-code pagination; in a real app read from query or signals

    let q: ProfileQuery = parse_query_params(cx).map_err(|e| {
        // turn parse failure into a 400 (adjust to your preferred error helper)
        topcoat::router::error::bad_request(format!("invalid query: {e}"))
    })?;
    let favourites = q.favourites();
    let page = q.page();
    let current_user = crate::auth::current_user(cx).await?;

    let current_user_id = if let Some(u) = current_user.clone() {
        Some(u.id)
    } else {
        None
    };

    let amount = match q.amount {
        Some(a) => a.clamp(1, 100),
        None => {
            if let Some(u) = current_user.as_ref() {
                crate::models::User::get(&u.username, cx)
                    .await
                    .map(|x| x.per_page_amount().clamp(1, 100))
                    .unwrap_or(10)
            } else {
                10
            }
        }
    };

    let articles = crate::models::article::Article::load_profile_articles(
        cx,
        &username,
        favourites,
        page,
        amount,
        current_user_id.expect("Should be id, else panic"),
    )
    .await?;

    let articles_count = articles.len();

    Ok(view! {
        <div class="mb-5">
            <div class="flex justify-between px-2 bg-gray-200 dark:bg-gray-900 py-2">
                <div class="flex text-gray-800 dark:text-gray-200">
                    user_articles_tab(user: username, active: !favourites)
                    favourited_articles_tab(user: username, active: favourites)
                </div> // items_per_page(...)
            </div>

            user_info(username: username)

            article_preview_list(articles: articles)

            <div class="flex justify-between">
                <div class="flex gap-4 mb-4">
                    // prev_next_button(...)
                    prev_next_profile(
                        page: page,
                        amount: amount,
                        favourites: favourites,
                        article_count: articles_count as i64,
                        username: username
                    )
                    <div class="flex gap-4 mt-4 px-2"></div>
                </div>
                <div class="flex justify-end px-7">back_to_home()</div>
            </div>
            // footer()
        </div>
    })
}

#[component]
async fn back_to_home() -> Result<impl View> {
    Ok(view! {
        <h4 class="text-blue-500 underline cursor-pointer">
            <a href="/">"Back to Home"</a>
        </h4>
    })
}

#[component]
async fn user_articles_tab(user: &str, active: bool) -> Result<impl View> {
    let class = if active {
        "font-bold border-b-8"
    } else {
        "font-bold cursor-pointer"
    };

    Ok(view! {
        <div class="mb-5 px-2">
            <a href=(format!("/profile/{}?favourites=false", user)) class=(class)>
                (user)
                "'s Articles"
            </a>
        </div>
    })
}

#[component]
async fn favourited_articles_tab(user: &str, active: bool) -> Result<impl View> {
    let class = if active {
        "font-bold border-b-8"
    } else {
        "font-bold cursor-pointer"
    };

    Ok(view! {
        <div class="mb-5 px-2">
            <a href=(format!("/profile/{}?favourites=true", user)) class=(class)>
                "Favourited Articles"
            </a>
        </div>
    })
}

#[component]
async fn user_info(cx: &Cx, username: &str) -> Result<impl View> {
    let user_profile = load_user_profile(cx, &username).await?;

    let image = user_profile.user.image();
    let name = user_profile.user.username();
    let bio = user_profile
        .user
        .bio()
        .unwrap_or_else(|| "No bio available".into());
    let email = if user_profile.user.email().is_empty() {
        " - ".to_string()
    } else {
        user_profile.user.email()
    };

    Ok(view! {
        <div
            class="bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 mb-2 p-4"
        >
            <div class="mb-5 px-5 flex justify-between">
                <h2 class="font-bold text-xl underline">
                    "Profile data of User - "
                    (name.clone())
                </h2>
                back_to_home()
            </div>

            <div class="flex">
                <div class="mb-4">
                    <img src=(image) class="w-10 h-10 rounded-full" />
                </div>
                <div class="px-4"><h4>(name)</h4></div>
            </div>

            <p>
                "Bio: "
                (bio)
            </p>
            <div>
                "Email: "
                (email)
            </div>
        </div>
    })
}

// ---------- helpers you will need ----------

// /// Replace with your real session / auth helper
// fn current_user_id(cx: &Cx) -> Option<i64> {
//     // e.g. try_request_context::<LoggedInUser>(cx).map(|u| u.id)
//     None
// }
