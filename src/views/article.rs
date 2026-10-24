use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Form,
        error::{bad_request, not_found},
        page, path_param, route,
    },
    view::{View, component, view},
};

use crate::database::db;
use crate::models::article::{Article, UserPreview}; // adjust paths to your crate

path_param!(slug: String, error = not_found);

// ---------------------------------------------------------------------------
// GET /article/{slug}
// ---------------------------------------------------------------------------

#[page("/article/{slug}")]
async fn article_page(cx: &Cx) -> Result<impl View> {
    // Owned path segment (avoids lifetime issues)
    let slug = path_param::<Slug>(cx)?.clone();

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

    let article = crate::models::article::Article::fetch_article(cx, &slug, user_id)
        .await
        .map_err(|_| not_found())?;

    let comments = crate::models::comment::fetch_comments(__cx, &slug)
        .await
        .unwrap_or_default();

    let is_owner = logged_user
        .as_ref()
        .map(|u| u == &article.author.username)
        .unwrap_or(false);

    Ok(view! {
        <div class="bg-opacity-60 inset-0 z-50 flex items-center justify-center py-6">
            <div
                class="z-70 block w-full max-w-6xl rounded-lg bg-white dark:bg-gray-800 p-4 shadow-lg mx-auto"
            >
                // Meta row
                article_meta(
                    article: article.clone()
                )

                // Favorite / Follow
                <div class="flex flex-wrap items-center gap-4 mb-5">
                    if logged_user.is_some() && logged_user.as_ref() != Some(&article.author.username) {
                        <form method="post" action=(format!("/article/{}/favorite", article.slug))>
                            <button
                                type="submit"
                                class=(if article.fav {
                                    "text-yellow-500 hover:text-gray-500 transition-colors duration-200"
                                } else {
                                    "text-gray-500 hover:text-yellow-500 transition-colors duration-200"
                                })
                            >
                                <i
                                    class=(if article.fav {
                                        "fas fa-star"
                                    } else {
                                        "far fa-star"
                                    })
                                ></i>
                                " Favourite"
                            </button>
                        </form>
                        <form method="post" action=(format!("/article/{}/follow", article.slug))>
                            <button
                                type="submit"
                                class=(if article.author.following {
                                    "text-yellow-500 hover:text-gray-400"
                                } else {
                                    "text-gray-500 hover:text-yellow-500"
                                })
                            >
                                <i
                                    class=(if article.author.following {
                                        "fa-solid fa-person-circle-minus"
                                    } else {
                                        "fa-solid fa-person-circle-plus"
                                    })
                                ></i>
                                " "
                                (if article.author.following {
                                    format!("Unfollow {}", article.author.username)
                                } else {
                                    format!("Follow {}", article.author.username)
                                })
                            </button>
                        </form>
                    }
                    <span class=(if article.favorites_count > 0 { "text-yellow-500" } else { "" })>
                        "Favourites: "
                        (article.favorites_count.to_string())
                    </span>
                </div>

                // Title + author + back
                <div class="flex justify-between mb-5">
                    <div>
                        <div class="mb-2">
                            <h1
                                class="text-xl leading-tight font-medium text-neutral-800 dark:text-gray-200"
                            >
                                (article.title.clone())
                            </h1>
                        </div>
                        author_user_icon(user: article.author.clone())
                    </div>
                    <div>
                        <a
                            href="/"
                            class="bg-blue-700 hover:bg-blue-800 px-4 py-2 text-white font-semibold rounded-lg"
                        >
                            "Back"
                        </a>
                    </div>
                </div>

                // Body
                <div class="mb-5">
                    <p class="text-neutral-800 dark:text-gray-300 whitespace-pre-wrap">
                        (article.body.clone().unwrap_or_default())
                    </p>
                </div>

                // Tags
                if !article.tag_list.is_empty() {
                    <div class="flex flex-wrap gap-1 mb-5">
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

                // Owner actions
                if is_owner {
                    <div class="flex gap-4 mb-5 text-sm">
                        <a
                            href=(format!("/editor/{}", article.slug))
                            class="text-blue-600 hover:underline"
                        >
                            <i class="fa-solid fa-pen-to-square"></i>
                            " Edit article"
                        </a>
                    </div>
                }

                // Comments
                comment_section(
                    slug: article.slug.clone(),
                    comments: comments,
                    logged_user: logged_user
                )

                // Bottom back
                <div class="flex justify-end mt-4">
                    <a
                        href="/"
                        class="bg-blue-700 hover:bg-blue-800 px-4 py-2 text-white font-semibold rounded-lg"
                    >
                        "Back"
                    </a>
                </div>
            </div>
        </div>
    })
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[component]
async fn article_meta(article: Article) -> Result<impl View> {
    Ok(view! {
        <div
            class="flex flex-wrap items-center gap-4 mb-4 text-gray-700 dark:text-gray-300"
        >
            <div class="flex items-center gap-1">
                <i class="fa-solid fa-calendar w-4 h-4"></i>
                <span>(article.created_at.clone())</span>
            </div>
            <div class="flex items-center gap-1">
                <i
                    class=(if article.comments_count > 0 {
                        "fas fa-comments w-4 h-4 text-yellow-500"
                    } else {
                        "far fa-comments w-4 h-4 text-gray-500"
                    })
                ></i>
                <span class="px-1">
                    "Comments: "
                    (article.comments_count.to_string())
                </span>
            </div>
            <div class="flex items-center gap-1">
                <i
                    class=(if article.fav {
                        "fas fa-heart text-red-500"
                    } else {
                        "far fa-heart"
                    })
                ></i>
                <span>(article.favorites_count.to_string())</span>
            </div>
        </div>
    })
}

#[component]
async fn author_user_icon(user: UserPreview) -> Result<impl View> {
    Ok(view! {
        <a
            href=(format!("/profile/{}", user.username))
            class="flex items-center gap-2 hover:underline text-gray-700 dark:text-gray-300"
        >
            if let Some(ref img) = user.image {
                <img
                    src=(img.clone())
                    alt=""
                    class="w-8 h-8 rounded-full object-cover"
                />
            } else {
                <i class="fa-regular fa-circle-user text-2xl"></i>
            }
            <span class="font-semibold">(user.username.clone())</span>
        </a>
    })
}

#[component]
async fn comment_section(
    slug: String,
    comments: Vec<crate::models::comment::Comment>,
    logged_user: Option<String>,
) -> Result<impl View> {
    let post_action = format!("/article/{}/comments", slug);

    Ok(view! {
        <div class="mb-1 border-t dark:border-gray-700 pt-4">
            <h2 class="mb-2 block text-sm font-bold text-gray-700 dark:text-gray-300">
                "Comments"
            </h2>

            // New comment form (POST)
            if logged_user.is_some() {
                <form method="post" action=(post_action) class="mb-5">
                    <div class="mb-1">
                        <textarea
                            class="focus:shadow-outline w-full border-b dark:border-gray-600 appearance-none rounded px-3 py-2 leading-tight text-sm text-gray-700 dark:text-gray-200 dark:bg-gray-700 shadow focus:outline-none"
                            name="body"
                            id="comment-body"
                            rows="3"
                            minlength="2"
                            required=""
                            placeholder="Write a new comment...(min 2 chars)"
                            oninput="document.getElementById('post-comment-btn').disabled = this.value.trim().length < 2;"
                        ></textarea>
                    </div>
                    <button
                        type="submit"
                        id="post-comment-btn"
                        disabled=""
                        class="rounded px-3 py-1 text-sm font-medium text-white bg-blue-700 hover:bg-blue-800 disabled:bg-gray-400 disabled:cursor-not-allowed"
                    >
                        "Post Comment"
                    </button>
                </form>
            } else {
                <p class="mb-4 text-sm text-gray-500">
                    <a href="/login" class="text-blue-600 hover:underline">"Log in"</a>
                    " to post a comment."
                </p>
            }

            // Comment list
            for comment in comments {
                comment_item(comment: comment, logged_user: logged_user.clone())
            }
        </div>
    })
}

#[component]
async fn comment_item(
    comment: crate::models::comment::Comment,
    logged_user: Option<String>,
) -> Result<impl View> {
    let can_delete = logged_user
        .as_ref()
        .map(|u| u == &comment.username)
        .unwrap_or(false);
    let delete_action = format!("/article/comments/{}/delete", comment.id);

    Ok(view! {
        <div class="py-5 border-b dark:border-gray-700">
            <div class="flex items-center gap-2 mb-2">
                if let Some(ref img) = comment.user_image {
                    <img
                        src=(img.clone())
                        alt=""
                        class="w-6 h-6 rounded-full object-cover"
                    />
                } else {
                    <i class="fa-regular fa-circle-user"></i>
                }
                <a
                    href=(format!("/profile/{}", comment.username))
                    class="font-semibold hover:underline"
                >
                    (comment.username.clone())
                </a>
            </div>
            <div class="flex grow justify-between gap-4">
                <p class="text-neutral-800 dark:text-gray-300">
                    (comment.body.clone())
                </p>
                <div class="flex-none px-3 text-gray-600 dark:text-gray-400 text-sm">
                    <div class="flex items-center gap-1 mb-1">
                        <i class="fa-solid fa-calendar w-4 h-4"></i>
                        <span class="px-1">(comment.created_at.clone())</span>
                    </div>
                    if can_delete {
                        <form method="post" action=(delete_action)>
                            <button
                                type="submit"
                                class="text-red-400 hover:rounded hover:border hover:bg-red-100 px-1"
                            >
                                <i class="fas fa-trash"></i>
                                <span class="px-1">"Delete"</span>
                            </button>
                        </form>
                    }
                </div>
            </div>
        </div>
    })
}

// ---------------------------------------------------------------------------
// POST handlers
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct NewCommentForm {
    body: String,
}

#[route(POST "/article/{slug}/comments")]
async fn post_comment(
    cx: &Cx,
    Form(form): Form<NewCommentForm>,
) -> Result<impl topcoat::router::response::IntoResponse> {
    let slug = path_param::<Slug>(cx)?.clone();

    // Resolve logged-in user from session
    let current_user = crate::auth::current_user(cx).await?;
    let logged_user = current_user.map(|u| u.id);
    let Some(user_id) = logged_user else {
        return Err(topcoat::router::error::unauthorized().into());
    };

    if form.body.trim().len() < 3 {
        return Err(topcoat::router::error::bad_request("comment too short").into());
    }

    crate::models::comment::insert_comment(cx, &slug, user_id, form.body.trim())
        .await
        .map_err(|e| bad_request(format!("could not post comment: {e}")))?;

    // Redirect back to the article
    Ok(topcoat::router::error::see_other(format!(
        "/article/{slug}"
    )))
}

#[route(POST "/article/{slug}/favorite")]
async fn favorite_article(cx: &Cx) -> Result<impl topcoat::router::response::IntoResponse> {
    let slug = path_param::<Slug>(cx)?.clone();

    let current_user = crate::auth::current_user(cx).await?;
    let Some(user_id) = current_user.map(|u| u.id) else {
        return Err(topcoat::router::error::unauthorized().into());
    };

    let exists: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM FavArticles WHERE article = ? AND user_id = ?")
            .bind(&slug)
            .bind(user_id)
            .fetch_optional(db(cx))
            .await
            .map_err(|e| bad_request(format!("db error: {e}")))?;

    if exists.is_some() {
        sqlx::query("DELETE FROM FavArticles WHERE article = ? AND user_id = ?")
            .bind(&slug)
            .bind(user_id)
            .execute(db(cx))
            .await
            .map_err(|e| bad_request(format!("db error: {e}")))?;
    } else {
        sqlx::query("INSERT INTO FavArticles(article, user_id) VALUES (?, ?)")
            .bind(&slug)
            .bind(user_id)
            .execute(db(cx))
            .await
            .map_err(|e| bad_request(format!("db error: {e}")))?;
    }

    Ok(topcoat::router::error::see_other(format!(
        "/article/{}",
        slug
    )))
}

#[route(POST "/article/{slug}/follow")]
async fn follow_author(cx: &Cx) -> Result<impl topcoat::router::response::IntoResponse> {
    let slug = path_param::<Slug>(cx)?.clone();

    let current_user = crate::auth::current_user(cx).await?;
    let Some(follower_id) = current_user.map(|u| u.id) else {
        return Err(topcoat::router::error::unauthorized().into());
    };

    let author_id: i64 = sqlx::query_scalar("SELECT author_id FROM Articles WHERE slug = ?")
        .bind(&slug)
        .fetch_one(db(cx))
        .await
        .map_err(|e| bad_request(format!("db error: {e}")))?;

    if author_id == follower_id {
        return Err(bad_request("cannot follow yourself").into());
    }

    let exists: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM Follows WHERE follower_id = ? AND influencer_id = ?")
            .bind(follower_id)
            .bind(author_id)
            .fetch_optional(db(cx))
            .await
            .map_err(|e| bad_request(format!("db error: {e}")))?;

    if exists.is_some() {
        sqlx::query("DELETE FROM Follows WHERE follower_id = ? AND influencer_id = ?")
            .bind(follower_id)
            .bind(author_id)
            .execute(db(cx))
            .await
            .map_err(|e| bad_request(format!("db error: {e}")))?;
    } else {
        sqlx::query("INSERT INTO Follows(follower_id, influencer_id) VALUES (?, ?)")
            .bind(follower_id)
            .bind(author_id)
            .execute(db(cx))
            .await
            .map_err(|e| bad_request(format!("db error: {e}")))?;
    }

    Ok(topcoat::router::error::see_other(format!(
        "/article/{}",
        slug
    )))
}

path_param!(id: i64, error = bad_request);

#[route(POST "/article/comments/{id}/delete")]
async fn delete_comment_route(cx: &Cx) -> Result<impl topcoat::router::response::IntoResponse> {
    let id = *path_param::<Id>(cx)?;

    let current_user = crate::auth::current_user(cx).await?;
    let logged_user = current_user.map(|u| u.id);
    let Some(user_id) = logged_user else {
        return Err(topcoat::router::error::unauthorized().into());
    };

    let slug = crate::models::comment::delete_comment(cx, id, user_id)
        .await
        .map_err(|e| bad_request(format!("could not delete: {e}")))?;

    let redirect_to = slug
        .map(|s| format!("/article/{s}"))
        .unwrap_or_else(|| "/".to_string());

    Ok(topcoat::router::error::see_other(redirect_to))
}
