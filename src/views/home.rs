use crate::database::db;
use crate::models::MatchedArticles;
use topcoat::{
    Result,
    context::Cx,
    router::{page, parse_query_params, query_params},
    view::{Unescaped, View, component, view},
};

use crate::components::tooltip::{tooltip, tooltip_content};
use crate::components::{article_preview::article_preview_list, prev_next_button::prev_next};
use crate::views::article::comment_section;
use topcoat::runtime::{shard, signal};
use topcoat::view::ViewExt;

#[query_params]
struct HomeQuery {
    page: Option<i64>,
    amount: Option<i64>,
    tag: Option<String>,
    my_feed: Option<bool>,
    search: Option<String>,
}

impl HomeQuery {
    fn page(&self) -> i64 {
        self.page.unwrap_or(0).max(0)
    }
    fn tag(&self) -> String {
        self.tag.as_deref().unwrap_or("").to_string()
    }
    fn my_feed(&self) -> bool {
        self.my_feed.unwrap_or(false)
    }
    fn search(&self) -> String {
        self.search.as_deref().unwrap_or("").trim().to_string()
    }
}

// ---------------------------------------------------------------------------
// HOME PAGE
// ---------------------------------------------------------------------------

#[page("/")]
pub async fn home(cx: &Cx) -> Result<impl View> {
    let q: HomeQuery = parse_query_params(cx)
        .map_err(|e| topcoat::router::error::bad_request(format!("invalid query: {e}")))?;
    let page = q.page();
    let tag = q.tag();
    let my_feed = q.my_feed();
    let search_query = q.search();
    let search_mode = !search_query.is_empty();

    let current_user = crate::auth::current_user(cx).await?;

    let logged_user = if let Some(u) = current_user.clone() {
        Some(u.username)
    } else {
        None
    };

    let user_id = if let Some(u) = current_user.clone() {
        Some(u.id)
    } else {
        None
    };

    // Default items-per-page to the logged-in user's persisted preference
    // when no explicit amount was provided in the query string.
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

    let epoch = signal(cx, || 1i32);
    let open_count = signal(cx, || 0i32);

    let tags = if search_mode {
        Vec::new()
    } else {
        fetch_popular_tags(cx).await.unwrap_or_default()
    };

    let articles = if search_mode {
        Vec::new()
    } else {
        crate::models::article::Article::load_home_articles(
            cx, page, amount, &tag, my_feed, user_id,
        )
        .await?
    };

    let (search_total, search_results) = if search_mode {
        let total = MatchedArticles::search_count(cx, &search_query)
            .await
            .unwrap_or(0);
        let results = MatchedArticles::search_articles(cx, &search_query, page, amount)
            .await
            .unwrap_or_default();
        (total, results)
    } else {
        (0, Vec::new())
    };

    Ok(view! {
        <div class="bg-gray-200 dark:bg-gray-900 min-h-screen">
            <div
                class="mx-auto sm:px-6 lg:px-8 px-2 py-2 text-gray-800 dark:text-gray-200"
            >
                // Top bar: feed tabs | search | items-per-page
                <div class="grid grid-cols-1 md:grid-cols-3 items-center gap-2 mb-2">
                    <div class="flex justify-start">
                        feed_tabs(
                            my_feed: my_feed,
                            logged_in: logged_user.is_some(),
                            amount: amount
                        )
                    </div>
                    <div class="flex items-center justify-center gap-3 flex-wrap">
                        // Search box
                        <form method="get" action="/" class="flex items-center gap-1">
                            <input type="hidden" name="page" value="0" />
                            <input
                                type="hidden"
                                name="amount"
                                value=(amount.to_string())
                            />
                            <input
                                class="shadow appearance-none bg-white dark:bg-gray-700 dark:text-white border dark:border-gray-600 rounded py-1 px-3 text-gray-700 leading-tight focus:outline-none"
                                type="text"
                                name="search"
                                minlength="2"
                                placeholder="Search string"
                                value=(search_query.clone())
                            />
                            tooltip(
                                <button type="submit" class="px-2 hover:text-blue-500">
                                    <i class="fas fa-magnifying-glass"></i>
                                </button>
                                tooltip_content("Search")
                            )
                        </form>
                        <div class="min-w-28">
                            if search_mode {
                                <a
                                    href="/"
                                    class="text-blue-400 hover:underline hover:text-blue-500 transition duration-200 cursor-pointer px-2"
                                >
                                    "Clear Search"
                                </a>
                            } else {
                                <span class="invisible">"Clear Search"</span>
                            }
                        </div>
                    </div>
                    <div class="flex justify-end">
                        items_per_page(
                            amount: amount,
                            tag: tag.clone(),
                            my_feed: my_feed,
                            search: search_query.clone()
                        )
                    </div>
                </div>

                if search_mode {
                    // Search results header
                    <div class="flex justify-between mb-1 flex-wrap gap-2">
                        <div class="font-bold">
                            "Search results = "
                            (search_total.to_string())
                        </div>
                        if search_total > 0 {
                            <div>
                                "  Showing results "
                                ({
                                    match page {
                                        0 => "1".to_string(),
                                        _ => (page * amount + 1).to_string(),
                                    }
                                })
                                " to "
                                ({
                                    match (page, search_total < amount) {
                                        (0, true) => search_total.to_string(),
                                        (0, false) => amount.to_string(),
                                        (_, _) => (std::cmp::min((page + 1) * amount, search_total)).to_string(

                                        ),
                                    }
                                })
                                " of "
                                (search_total.to_string())
                            </div>
                        }
                    </div>

                    #[key(article.slug.clone())]
                    for article in search_results.iter() {
                        search_result_item(
                            article: article.clone(),
                            epoch: epoch.clone(),
                            open_count: open_count.clone()
                        )
                    }

                    // Prev / Next for search
                    <div class="flex gap-4 mt-4 px-2 justify-end">
                        if page > 0 {
                            <a
                                href=(format!(
                                    "/?search={}&page={}&amount={}",
                                    urlencoding(search_query.clone()),
                                    (page - 1).max(0),
                                    amount,
                                ))
                                class="px-4 cursor-pointer hover:text-blue-500 border dark:border-gray-600 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200"
                            >
                                "<< Previous page"
                            </a>
                        }
                        if search_results.len() as i64 >= amount
                            && search_total > (page + 1) * amount {
                            <a
                                href=(format!(
                                    "/?search={}&page={}&amount={}",
                                    urlencoding(search_query.clone()),
                                    page + 1,
                                    amount,
                                ))
                                class="px-4 cursor-pointer hover:text-blue-500 border dark:border-gray-600 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200"
                            >
                                "Next page >>"
                            </a>
                        }
                    </div>
                } else {
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
                }
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
async fn items_per_page(
    amount: i64,
    tag: String,
    my_feed: bool,
    search: String,
) -> Result<impl View> {
    Ok(view! {
        <form method="get" action="/" class="flex items-center gap-1">
            <input type="hidden" name="page" value="0" />
            <input type="hidden" name="tag" value=(tag) />
            <input type="hidden" name="my_feed" value=(my_feed.to_string()) />
            <input type="hidden" name="search" value=(search) />
            <label class="text-gray-700 dark:text-gray-300 px-1" for="articlesPerPage">
                "Items Per Page"
            </label>
            <select
                id="articlesPerPage"
                name="amount"
                class="focus:shadow-outline rounded border dark:border-gray-600 px-1 py-1 leading-tight text-gray-700 dark:text-gray-200 dark:bg-gray-700 shadow focus:outline-none"
                onchange="const sel=this; fetch('/items-per-page',{method:'POST',credentials:'same-origin',headers:{'Content-Type':'application/x-www-form-urlencoded'},body:'amount='+encodeURIComponent(sel.value)}).finally(()=>sel.form.submit());"
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

// ---- Search result item ----

#[component]
async fn search_result_item(
    cx: &Cx,
    article: MatchedArticles,
    epoch: topcoat::runtime::Signal<i32>,
    open_count: topcoat::runtime::Signal<i32>,
) -> Result<impl View> {
    let slug = article.slug.clone();
    let title = article.title.clone().unwrap_or_default();
    let description = article.description.clone().unwrap_or_default();
    let body = article.body.clone().unwrap_or_default();

    let item_epoch = signal(cx, || 0i32);

    Ok(view! {
        <div
            class="mb-2 p-4 bg-white dark:bg-gray-800 rounded-lg shadow-md text-gray-800 dark:text-gray-200"
        >
            <p>
                <span class="font-bold">"Title: "</span>
                <span>(Unescaped::new_unchecked(title))</span>
            </p>
            <p>
                <span class="font-bold">"Description: "</span>
                <span>(Unescaped::new_unchecked(description))</span>
            </p>
            <p>
                <span class="font-bold">"Body: "</span>
                <span>(Unescaped::new_unchecked(body))</span>
            </p>
            <div class="flex justify-between mt-2">
                <div>
                    <a
                        href=(format!("/article/{}", slug))
                        class="text-blue-600 underline cursor-pointer"
                        target="_blank"
                    >
                        "Open in a new tab/window"
                    </a>
                </div>
                <div class="flex items-center gap-3">
                    <button
                        class="text-blue-600 underline cursor-pointer"
                        type="button"
                        @click=$(|_e| {
                            let currently_open = if item_epoch.get() == 0i32 {
                                false
                            } else {
                                item_epoch.get() == epoch.get()
                            };
                            if currently_open {
                                item_epoch.set(0i32);
                                let c = open_count.get();
                                if c > 0i32 {
                                    open_count.set(c - 1i32);
                                }
                            } else {
                                item_epoch.set(epoch.get());
                                open_count.set(open_count.get() + 1i32);
                            }
                        })
                    >
                        $(if item_epoch.get() == 0i32 {
                            "Show Full Article"
                        } else {
                            if item_epoch.get() == epoch.get() {
                                "Hide"
                            } else {
                                "Show Full Article"
                            }
                        })
                    </button>
                    <div
                        :hidden=$(if open_count.get() > 1i32 {
                            if item_epoch.get() == 0i32 {
                                true
                            } else {
                                if item_epoch.get() == epoch.get() { false } else { true }
                            }
                        } else {
                            true
                        })
                    >
                        <button
                            class="text-blue-600 underline cursor-pointer"
                            type="button"
                            @click=$(|_e| {
                                epoch.set(epoch.get() + 1i32);
                                open_count.set(0i32);
                            })
                        >
                            "Hide All"
                        </button>
                    </div>
                </div>
            </div>
            <div
                :hidden=$(if item_epoch.get() == 0i32 {
                    true
                } else {
                    if item_epoch.get() == epoch.get() { false } else { true }
                })
                class="mt-3"
            >
                search_article_view(
                    slug: slug.clone(),
                    show: $(if item_epoch.get() == 0i32 {
                        false
                    } else {
                        item_epoch.get() == epoch.get()
                    })
                )
            </div>
        </div>
    })
}

#[shard]
async fn search_article_view(cx: &Cx, slug: String, show: bool) -> Result<impl View> {
    if !show {
        return Ok(view! { <div></div> }.boxed());
    }

    let current_user = crate::auth::current_user(cx).await?;
    let user_id = current_user.as_ref().map(|u| u.id);
    let logged_user = current_user.map(|u| u.username);

    let article = crate::models::article::Article::fetch_article(cx, &slug, user_id)
        .await
        .map_err(|_| topcoat::router::error::not_found())?;

    let comments = crate::models::comment::fetch_comments(cx, &slug)
        .await
        .unwrap_or_default();

    let is_owner = logged_user
        .as_ref()
        .map(|u| u == &article.author.username)
        .unwrap_or(false);

    Ok(view! {
        <div class="mt-3 border-t dark:border-gray-700 pt-3">
            <h2 class="text-xl font-bold mb-2 text-gray-800 dark:text-gray-200">
                (article.title.clone())
            </h2>
            <p class="text-gray-700 dark:text-gray-300 whitespace-pre-wrap mb-3">
                (article.body.clone().unwrap_or_default())
            </p>
            if !article.tag_list.is_empty() {
                <div class="flex flex-wrap gap-1 mb-3">
                    <i class="fa-solid fa-hashtag py-1 text-gray-500"></i>
                    for t in article.tag_list.clone() {
                        if !t.is_empty() {
                            <a
                                href=(format!("/?page=0&amount=10&tag={t}&my_feed=false"))
                                class="bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 px-2 py-1 rounded text-xs"
                            >
                                (t)
                            </a>
                        }
                    }
                </div>
            }
            if is_owner {
                <div class="mb-3 text-sm">
                    <a
                        href=(format!("/editor/{}", article.slug))
                        class="text-blue-600 hover:underline"
                    >
                        "Edit article"
                    </a>
                </div>
            }
            comment_section(
                slug: article.slug.clone(),
                comments: comments,
                logged_user: logged_user
            )
        </div>
    }.boxed())
}

// ---------------------------------------------------------------------------
// DB helpers
// ---------------------------------------------------------------------------

async fn fetch_popular_tags(cx: &Cx) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            tag,
            tag_count,
            max_created_at
        FROM (
            SELECT
                T1.tag,
                COUNT(T1.article) AS tag_count,
                MAX(T2.created_at) AS max_created_at
            FROM
                ArticleTags AS T1
            JOIN
                Articles AS T2
            ON
                T1.article = T2.slug
            GROUP BY
                T1.tag
            ORDER BY
                tag_count DESC
            LIMIT 10
        )

        UNION

        SELECT
            tag,
            tag_count,
            max_created_at
        FROM (
            SELECT
                T1.tag,
                COUNT(T1.article) AS tag_count,
                MAX(T2.created_at) AS max_created_at
            FROM
                ArticleTags AS T1
            JOIN
                Articles AS T2
            ON
                T1.article = T2.slug
            GROUP BY
                T1.tag
            ORDER BY
                max_created_at DESC
            LIMIT 10
        )
        "#
    )
    .fetch_all(db(cx))
    .await?;

    Ok(rows.into_iter().filter_map(|r| r.tag).collect())
}

fn urlencoding(s: String) -> String {
    // minimal percent-encoding for query values
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
