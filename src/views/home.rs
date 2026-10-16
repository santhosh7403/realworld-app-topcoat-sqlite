use crate::database::db;
use topcoat::{
    Result,
    context::Cx,
    router::{page, parse_query_params, query_params},
    view::{View, component, view},
};

use crate::components::{article_preview::article_preview_list, prev_next_button::prev_next};

#[query_params]
struct HomeQuery {
    page: Option<i64>,
    amount: Option<i64>,
    tag: Option<String>,
    my_feed: Option<bool>,
}

impl HomeQuery {
    fn page(&self) -> i64 {
        self.page.unwrap_or(0).max(0)
    }
    fn amount(&self) -> i64 {
        self.amount.unwrap_or(10).clamp(1, 100)
    }
    fn tag(&self) -> String {
        self.tag.as_deref().unwrap_or("").to_string()
    }
    fn my_feed(&self) -> bool {
        self.my_feed.unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// HOME PAGE
// ---------------------------------------------------------------------------

#[page("/")]
pub async fn home(cx: &Cx) -> Result<impl View> {
    // let q = query_params::<HomeQuery>(cx)?;
    let q: HomeQuery = parse_query_params(cx).map_err(|e| {
        // turn parse failure into a 400 (adjust to your preferred error helper)
        topcoat::router::error::bad_request(format!("invalid query: {e}"))
    })?;
    let page = q.page();
    let amount = q.amount();
    let tag = q.tag();
    let my_feed = q.my_feed();
    let current_user = crate::auth::current_user(cx).await?;

    let logged_user = if let Some(u) = current_user.clone() {
        Some(u.username)
    } else {
        None
    }; // wire to session later

    let user_id = if let Some(u) = current_user {
        Some(u.id)
    } else {
        None
    };

    // logged-in user: stub for now (wire to session later)
    // let logged_user: Option<String> = None;

    // let (articles, tags) = match try_get_db(cx) {
    //     Some(db_pool) => {
    //         let db = db_pool.clone();
    //         let articles = crate::models::article::Article::load_home_articles(
    //             cx, page, amount, &tag, my_feed, user_id,
    //         )
    //         .await
    //         .unwrap_or_default();
    //         let tags = fetch_popular_tags(&db).await.unwrap_or_default();
    //         (articles, tags)
    //     }
    //     None => (Vec::new(), Vec::new()),
    // };

    let articles = crate::models::article::Article::load_home_articles(
        cx, page, amount, &tag, my_feed, user_id,
    )
    .await?;
    let tags = fetch_popular_tags(cx).await.unwrap_or_default();

    Ok(view! {
        <div class="bg-gray-200 dark:bg-gray-900 min-h-screen">
            <div
                class="mx-auto sm:px-6 lg:px-8 px-2 py-2 text-gray-800 dark:text-gray-200"
            >
                // Top bar: feed tabs | search | items-per-page
                <div class="flex flex-wrap justify-between items-center gap-2 mb-2">
                    <div class="flex">
                        feed_tabs(
                            my_feed: my_feed,
                            logged_in: logged_user.is_some(),
                            amount: amount
                        )
                    </div>
                    <div class="flex items-center gap-3">
                        <form method="get" action="/" class="flex">
                            <input type="hidden" name="page" value="0" />
                            <input
                                type="hidden"
                                name="amount"
                                value=(amount.to_string())
                            />
                            <input
                                type="hidden"
                                name="my_feed"
                                value=(my_feed.to_string())
                            />
                            <input
                                class="shadow appearance-none bg-white dark:bg-gray-700 dark:text-white border dark:border-gray-600 rounded py-1 px-3 text-gray-700 leading-tight focus:outline-none"
                                type="text"
                                name="tag"
                                minlength="2"
                                placeholder="Filter by tag…"
                                value=(tag.clone())
                            />
                            <button type="submit" class="px-2 hover:text-blue-500">
                                <i class="fas fa-magnifying-glass"></i>
                            </button>
                        </form>
                        items_per_page(
                            amount: amount,
                            tag: tag.clone(),
                            my_feed: my_feed
                        )
                    </div>
                </div>

                // Popular tags (only on Global Feed)
                if !my_feed {
                    <div
                        class="flex gap-1 rounded bg-white dark:bg-gray-800 mb-2 p-1 flex-wrap items-center"
                    >
                        <span class="font-bold m-1">"Popular Tags:"</span>
                        tag_list(
                            tags: tags,
                            current_tag: tag.clone(),
                            amount: amount,
                            my_feed: my_feed
                        )
                    </div>
                }

                // Article list
                if articles.is_empty() {
                    if my_feed {
                        <p class="text-gray-700 dark:text-gray-300 p-4">
                            "You are not following any other user!"
                        </p>
                    } else {
                        <p class="text-gray-700 dark:text-gray-300 p-4">
                            "No articles to list"
                        </p>
                    }
                } else {
                    article_preview_list(articles: articles.clone())
                }

                // Prev / Next
                <div class="flex gap-4 mt-4 px-2">
                    prev_next(
                        page: page,
                        amount: amount,
                        tag: tag.clone(),
                        my_feed: my_feed,
                        article_count: articles.len() as i64
                    )
                </div>
            </div>
        </div>
    })
}

// ---- Feed tabs ----

#[component]
async fn feed_tabs(my_feed: bool, logged_in: bool, amount: i64) -> Result<impl View> {
    let your_feed_href = format!("/?page=0&amount={amount}&tag=&my_feed=true");
    let global_feed_href = format!("/?page=0&amount={amount}&tag=&my_feed=false");

    let your_cls = if !logged_in {
        "px-1 m-1 font-bold cursor-not-allowed bg-gray-200 dark:bg-gray-700 dark:text-gray-400"
    } else if my_feed {
        "px-1 m-1 font-bold border-b-8 bg-gray-200 dark:bg-gray-700 dark:text-white border-blue-500"
    } else {
        "px-1 m-1 font-bold bg-gray-200 dark:bg-gray-800 dark:text-gray-300 cursor-pointer hover:dark:bg-gray-700"
    };

    let global_cls = if !my_feed {
        "px-1 m-1 font-bold border-b-8 bg-gray-200 dark:bg-gray-700 dark:text-white border-blue-500"
    } else {
        "px-1 m-1 font-bold bg-gray-200 dark:bg-gray-800 dark:text-gray-300 cursor-pointer hover:dark:bg-gray-700"
    };

    Ok(view! {
        if logged_in {
            <a href=(your_feed_href) class=(your_cls)>"Your Feed"</a>
        } else {
            <span class=(your_cls)>"Your Feed"</span>
        }
        <a href=(global_feed_href) class=(global_cls)>"Global Feed"</a>
    })
}

// ---- Items per page ----

#[component]
async fn items_per_page(amount: i64, tag: String, my_feed: bool) -> Result<impl View> {
    Ok(view! {
        <form method="get" action="/" class="flex items-center gap-1">
            <input type="hidden" name="page" value="0" />
            <input type="hidden" name="tag" value=(tag) />
            <input type="hidden" name="my_feed" value=(my_feed.to_string()) />
            <label class="text-gray-700 dark:text-gray-300 px-1" for="articlesPerPage">
                "Items Per Page"
            </label>
            <select
                id="articlesPerPage"
                name="amount"
                class="focus:shadow-outline rounded border dark:border-gray-600 px-1 py-1 leading-tight text-gray-700 dark:text-gray-200 dark:bg-gray-700 shadow focus:outline-none"
                onchange="this.form.submit()"
            >
                <option value="1" selected=(amount == 1)>"1"</option>
                <option value="5" selected=(amount == 5)>"5"</option>
                <option value="10" selected=(amount == 10)>"10"</option>
                <option value="20" selected=(amount == 20)>"20"</option>
                <option value="100" selected=(amount == 100)>"100"</option>
            </select>
        </form>
    })
}

// ---- Tag list ----

#[component]
async fn tag_list(
    tags: Vec<String>,
    current_tag: String,
    amount: i64,
    my_feed: bool,
) -> Result<impl View> {
    Ok(view! {
        <div class="flex gap-1 py-1 flex-wrap">
            for tag in tags {
                {
                    // let is_selected = tag == current_tag;
                    // let href = if is_selected {
                    //     format!("/?page=0&amount={amount}&tag=&my_feed={my_feed}")
                    // } else {
                    //     format!("/?page=0&amount={amount}&tag={tag}&my_feed={my_feed}")
                    // };
                    // let cls = if is_selected {
                    //     "rounded px-1 py-0.5 bg-green-200 dark:bg-green-800 text-gray-800 dark:text-gray-200"
                    // } else {
                    //     "rounded px-1 py-0.5 bg-gray-200 dark:bg-gray-700 hover:bg-green-300 dark:hover:bg-green-700 text-gray-800 dark:text-gray-200"
                    // };
                    // view! {
                    //     <a href=(href) class=(cls)>(tag)</a>
                    // }
                    tag_link(
                        tag: tag,
                        current_tag: current_tag.clone(),
                        amount: amount,
                        my_feed: my_feed
                    )
                }
            }
        </div>
    })
}

#[component]
async fn tag_link(
    tag: String,
    current_tag: String,
    amount: i64,
    my_feed: bool,
) -> Result<impl View> {
    let is_selected = tag == current_tag;
    let href = if is_selected {
        format!("/?page=0&amount={amount}&tag=&my_feed={my_feed}")
    } else {
        format!("/?page=0&amount={amount}&tag={tag}&my_feed={my_feed}")
    };
    let cls = if is_selected {
        "rounded px-1 py-0.5 bg-green-200 dark:bg-green-800 text-gray-800 dark:text-gray-200"
    } else {
        "rounded px-1 py-0.5 bg-gray-200 dark:bg-gray-700 hover:bg-green-300 dark:hover:bg-green-700 text-gray-800 dark:text-gray-200"
    };

    Ok(view! { <a href=(href) class=(cls)>(tag)</a> })
}

// ---- Article preview list ----

// #[component]
// pub async fn article_preview_list(articles: Vec<Article>) -> Result<impl View> {
//     Ok(view! {
//         for article in articles {
//             article_preview(article: article)
//         }
//     })
// }

// #[component]
// async fn article_preview(article: Article) -> Result<impl View> {
//     let href = format!("/article/{}", article.slug);
//     Ok(view! {
//         <div class="mb-2 p-4 bg-white dark:bg-gray-800 rounded-lg shadow-md">
//             <div class="flex items-center gap-4 mb-4 text-gray-700 dark:text-gray-300">
//                 <a
//                     href=(format!("/profile/{}", article.author.username))
//                     class="flex items-center gap-2 hover:underline"
//                 >
//                     if let Some(ref img) = article.author.image {
//                         <img
//                             src=(img.clone())
//                             alt=""
//                             class="w-8 h-8 rounded-full object-cover"
//                         />
//                     } else {
//                         <i class="fa-regular fa-circle-user text-2xl"></i>
//                     }
//                     <span class="font-semibold">(article.author.username.clone())</span>
//                 </a>
//                 <div class="flex items-center gap-1">
//                     <i class="fa-solid fa-calendar w-4 h-4"></i>
//                     <span>(article.created_at.clone())</span>
//                 </div>
//                 <div class="flex items-center gap-1">
//                     <i
//                         class=(if article.comments_count > 0 {
//                             "fas fa-comments w-4 h-4 text-yellow-500"
//                         } else {
//                             "far fa-comments w-4 h-4 text-gray-500"
//                         })
//                     ></i>
//                     <span class="px-1">
//                         "Comments: "
//                         (article.comments_count.to_string())
//                     </span>
//                 </div>
//                 <div class="flex items-center gap-1">
//                     <i
//                         class=(if article.fav {
//                             "fas fa-heart text-red-500"
//                         } else {
//                             "far fa-heart"
//                         })
//                     ></i>
//                     <span>(article.favorites_count.to_string())</span>
//                 </div>
//             </div>

//             <a href=(href.clone())>
//                 <h2 class="text-2xl font-bold mb-2 text-gray-800 dark:text-gray-200">
//                     (article.title.clone())
//                 </h2>
//             </a>
//             <a href=(href.clone())>
//                 <p class="text-gray-700 dark:text-gray-300 mb-4">
//                     (article.description.clone())
//                 </p>
//             </a>

//             <div
//                 class="flex justify-between items-end text-gray-600 dark:text-gray-400"
//             >
//                 <a href=(href) class="hover:text-blue-600 hover:underline">
//                     "Read more..."
//                 </a>
//                 <div class="flex flex-wrap gap-1 items-center">
//                     <i class="fa-solid fa-hashtag py-1"></i>
//                     for t in article.tag_list {
//                         if !t.is_empty() {
//                             <a
//                                 href=(format!("/?page=0&amount=10&tag={t}&my_feed=false"))
//                                 class="bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 px-2 py-1 rounded text-xs hover:bg-green-200 dark:hover:bg-green-800"
//                             >
//                                 (t)
//                             </a>
//                         }
//                     }
//                 </div>
//             </div>
//         </div>
//     })
// }

// ---- Prev / Next ----

// #[component]
// pub async fn prev_next(
//     page: i64,
//     amount: i64,
//     tag: String,
//     my_feed: bool,
//     article_count: i64,
// ) -> Result<impl View> {
//     let prev_href = format!(
//         "/?page={}&amount={}&tag={}&my_feed={}",
//         (page - 1).max(0),
//         amount,
//         tag,
//         my_feed
//     );
//     let next_href = format!(
//         "/?page={}&amount={}&tag={}&my_feed={}",
//         page + 1,
//         amount,
//         tag,
//         my_feed
//     );
//     let btn = "px-4 cursor-pointer hover:text-blue-500 border dark:border-gray-600 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200";

//     Ok(view! {
//         if page > 0 {
//             <a href=(prev_href) class=(btn)>"<< Previous page"</a>
//         }
//         if article_count > 0 && article_count >= amount {
//             <a href=(next_href) class=(btn)>"Next page >>"</a>
//         }
//     })
// }

// ---------------------------------------------------------------------------
// DB helpers
// ---------------------------------------------------------------------------

// pub fn try_get_db(cx: &Cx) -> Option<&SqlitePool> {
//     topcoat::context::try_app_context::<AppState>(cx).map(|s| &s.db)
// }

// async fn fetch_home_articles(
//     db: &SqlitePool,
//     page: i64,
//     amount: i64,
//     tag: &str,
//     my_feed: bool,
//     user_id: Option<i64>,
// ) -> Result<Vec<Article>, sqlx::Error> {
//     let offset = page * amount;
//     // let username = username.unwrap_or("");

//     sqlx::query!(
//         "
// SELECT
// a.slug,
// a.title,
// a.description,
// a.created_at,
// (SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
// (SELECT COUNT(*) FROM comments WHERE article=a.slug) as comments_count,
// u.username, u.image,
// EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and user_id=$5) as fav,
// EXISTS(SELECT 1 FROM Follows WHERE follower_id=$5 and influencer_id=u.id) as following,
// (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
// FROM Articles as a
// JOIN Users as u ON a.author_id = u.id
// WHERE
// CASE WHEN $3!='' THEN a.slug in (SELECT distinct article FROM ArticleTags WHERE tag=$3)
// ELSE 1=1
// END
// AND
// CASE WHEN $4 THEN u.id in (SELECT influencer_id FROM Follows WHERE follower_id=$5)
// ELSE 1=1
// END
// ORDER BY a.created_at desc
// LIMIT $1 OFFSET $2",
//         amount,
//         offset,
//         tag,
//         my_feed,
//         user_id,
//     )
//     .map(|x| Article {
//         slug: x.slug,
//         title: x.title,
//         body: None, // no need
//         fav: x.fav != 0,
//         // fav: x.fav.unwrap_or_default(),
//         description: x.description,
//         created_at: x.created_at.format(crate::models::DATE_FORMAT).to_string(),
//         favorites_count: x.favorites_count,
//         // favorites_count: x.favorites_count.unwrap_or_default(),
//         author: UserPreview {
//             username: x.username,
//             image: x.image,
//             following: x.following != 0,
//             // following: x.following.unwrap_or_default(),
//         },
//         tag_list: x
//             .tag_list
//             .unwrap_or_default()
//             .split(' ')
//             .map(ToString::to_string)
//             .collect::<Vec<String>>(),
//         comments_count: x.comments_count,
//         // comments_count: x.comments_count.unwrap_or_default(),
//     })
//     .fetch_all(db)
//     .await
// }

async fn fetch_popular_tags(cx: &Cx) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
SELECT tag
FROM ArticleTags
GROUP BY tag
ORDER BY COUNT(*) DESC
LIMIT 20
        "#
    )
    .fetch_all(db(cx))
    .await?;

    Ok(rows.into_iter().map(|r| r.tag).collect())
}
