use sqlx::SqlitePool;
use std::collections::HashSet;
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{
        content::Form,
        error::{RouterErrorExt, bad_request, internal_server_error, see_other, unauthorized},
        page, path_param, route,
    },
    view::{View, component, view},
};
use uuid::Uuid;

const TITLE_MIN_LENGTH: usize = 4;
const DESCRIPTION_MIN_LENGTH: usize = 4;
const BODY_MIN_LENGTH: usize = 10;

#[derive(Debug, serde::Deserialize)]
pub struct ArticleForm {
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: String,
    /// Empty = create, non-empty = update
    #[serde(default)]
    pub slug: String,
}

struct ArticleUpdate {
    title: String,
    description: String,
    body: String,
    tag_list: HashSet<String>,
}

// ---------------------------------------------------------------------------
// Validation + DB
// ---------------------------------------------------------------------------

fn validate_article(form: &ArticleForm) -> Result<ArticleUpdate, String> {
    if form.title.len() < TITLE_MIN_LENGTH {
        return Err("You need to provide a title with at least 4 characters".into());
    }
    if form.description.len() < DESCRIPTION_MIN_LENGTH {
        return Err("You need to provide a description with at least 4 characters".into());
    }
    if form.body.len() < BODY_MIN_LENGTH {
        return Err("You need to provide a body with at least 10 characters".into());
    }

    let tag_list = form
        .tag_list
        .trim()
        .split_ascii_whitespace()
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect::<HashSet<_>>();

    Ok(ArticleUpdate {
        title: form.title.clone(),
        description: form.description.clone(),
        body: form.body.clone(),
        tag_list,
    })
}

async fn update_article(
    db: &SqlitePool,
    author_id: i64,
    slug: &str,
    article: ArticleUpdate,
) -> Result<String, sqlx::Error> {
    static BIND_LIMIT: usize = 65535;

    let mut tx = db.begin().await?;

    let (rows_affected, new_slug) = if !slug.is_empty() {
        let result = sqlx::query!(
            r#"
            UPDATE Articles
            SET title = $1, description = $2, body = $3
            WHERE slug = $4 AND author_id = $5
            "#,
            article.title,
            article.description,
            article.body,
            slug,
            author_id,
        )
        .execute(&mut *tx)
        .await?;

        (result.rows_affected(), slug.to_string())
    } else {
        let new_slug = Uuid::now_v7().to_string();
        let result = sqlx::query!(
            r#"
            INSERT INTO Articles (slug, title, description, body, author_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            new_slug,
            article.title,
            article.description,
            article.body,
            author_id,
        )
        .execute(&mut *tx)
        .await?;

        (result.rows_affected(), new_slug)
    };

    if rows_affected != 1 {
        return Err(sqlx::Error::RowNotFound);
    }

    sqlx::query!("DELETE FROM ArticleTags WHERE article = $1", new_slug)
        .execute(&mut *tx)
        .await?;

    if !article.tag_list.is_empty() {
        let mut qb = sqlx::QueryBuilder::new("INSERT INTO ArticleTags (article, tag) ");
        qb.push_values(
            article.tag_list.into_iter().take(BIND_LIMIT / 2),
            |mut b, tag| {
                b.push_bind(new_slug.clone()).push_bind(tag);
            },
        );
        qb.build().execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(new_slug)
}

// ---------------------------------------------------------------------------
// Shared component – the actual editor form
// ---------------------------------------------------------------------------

#[component]
async fn article_editor(
    slug: &str,
    title: &str,
    description: &str,
    body: &str,
    tag_list: &str,
    error: Option<&str>,
) -> Result<impl View> {
    Ok(view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-gray-900 bg-opacity-60"
        >
            <div
                class="block rounded-lg bg-white dark:bg-gray-800 w-full max-w-md p-4 shadow-[0_2px_15px_-3px_rgba(0,0,0,0.07),0_10px_20px_-2px_rgba(0,0,0,0.04)] z-70"
            >
                <div class="col-md-10 offset-md-1 col-xs-12">
                    <form method="post" action="/editor">
                        <input type="hidden" name="slug" value=(slug)>

                        if let Some(msg) = error {
                            <div class="mb-4 text-red-600">(msg)</div>
                        }

                        <div class="mb-5">
                            <input
                                name="title"
                                type="text"
                                class="shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring"
                                placeholder="Article Title"
                                minlength=(TITLE_MIN_LENGTH)
                                value=(title)
                                required=""
                            >
                        </div>

                        <div class="mb-5">
                            <input
                                name="description"
                                type="text"
                                class="shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring"
                                placeholder="What's this article about?"
                                minlength=(DESCRIPTION_MIN_LENGTH)
                                value=(description)
                                required=""
                            >
                        </div>

                        <div class="mb-5">
                            <textarea
                                name="body"
                                rows="8"
                                class="shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring"
                                placeholder="Write your article (in markdown)"
                                minlength=(BODY_MIN_LENGTH)
                                required=""
                            >
                                (body)
                            </textarea>
                        </div>

                        <div class="mb-5">
                            <input
                                name="tag_list"
                                type="text"
                                class="shadow appearance-none border dark:border-gray-600 rounded w-full py-2 px-3 text-gray-700 dark:text-gray-200 dark:bg-gray-700 leading-tight focus:ring"
                                placeholder="Enter tags (space separated)"
                                value=(tag_list)
                            >
                        </div>

                        <div class="flex flex-wrap flex-row-reverse gap-4">
                            <button
                                type="submit"
                                class="bg-blue-700 hover:bg-blue-800 px-5 py-3 text-white rounded-lg"
                            >
                                "Publish Article"
                            </button>

                            <a
                                href="/"
                                class="bg-gray-300 hover:bg-gray-400 dark:bg-gray-600 dark:hover:bg-gray-500 px-5 py-3 text-white rounded-lg"
                            >
                                "Cancel"
                            </a>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    })
}

// ---------------------------------------------------------------------------
// GET pages – thin wrappers that call the component
// ---------------------------------------------------------------------------

path_param!(slug);

/// Create new article
#[page("/new-article")]
async fn editor_new() -> Result<impl View> {
    Ok(view! {
        article_editor(
            slug: "",
            title: "",
            description: "",
            body: "",
            tag_list: "",
            error: None
        )
    })
}

/// Edit existing article
#[page("/editor/{slug}")]
async fn editor_edit(cx: &Cx) -> Result<impl View> {
    let slug = path_param::<Slug>(cx);

    let article = crate::models::article::Article::fetch_article(cx, slug, None)
        .await
        .ok_or_not_found()?;
    // .ok_or_not_found();
    // .unwrap();
    let tag_list = article.tag_list.join(" ");
    let body = article.body.unwrap_or_default();
    Ok(view! {
        article_editor(
            slug: slug,
            title: &article.title,
            description: &article.description,
            body: &body,
            tag_list: &tag_list,
            error: None
        )
    })
}

// ---------------------------------------------------------------------------
// POST route – create or update
// ---------------------------------------------------------------------------

#[route(POST "/editor")]
async fn editor_submit(
    cx: &Cx,
    Form(form): Form<ArticleForm>,
) -> Result<impl topcoat::router::response::IntoResponse> {
    let state = app_context::<crate::AppState>(cx);

    let current_user = crate::auth::current_user(cx).await?;

    let user_id = if let Some(u) = current_user {
        u.id
    } else {
        return Err(unauthorized().into());
    };
    // let user_id = current_user.unwrap_or(default)

    // Auth – replace with your real session/cookie helper
    // let author = get_current_username(cx).await.ok_or_else(unauthorized)?;

    let article = validate_article(&form).map_err(bad_request)?;

    let slug = update_article(&state.db, user_id, &form.slug, article)
        .await
        .map_err(|e| {
            tracing::error!("EDITOR ERROR: {e}");
            internal_server_error(e)
        })?;

    Ok(see_other(format!("/article/{slug}")))
}
