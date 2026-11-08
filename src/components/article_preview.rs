use crate::models::article::Article;
use topcoat::{
    Result,
    view::{View, component, view},
};

#[component]
pub async fn article_preview_list(articles: Vec<Article>) -> Result<impl View> {
    Ok(view! {
        for article in articles {
            article_preview(article: article)
        }
    })
}

#[component]
async fn article_preview(article: Article) -> Result<impl View> {
    let href = format!("/article/{}", article.slug);
    Ok(view! {
        <div class="mb-2 p-4 bg-white dark:bg-gray-800 rounded-lg shadow-md">
            <div class="flex flex-wrap items-center gap-4 mb-4 text-gray-700 dark:text-gray-300">
                <a
                    href=(format!("/profile/{}", article.author.username))
                    class="flex items-center gap-2 hover:underline"
                >
                    if let Some(ref img) = article.author.image {
                        <img
                            src=(img.clone())
                            alt=""
                            class="w-8 h-8 rounded-full object-cover"
                        />
                    } else {
                        <i class="fa-regular fa-circle-user text-2xl"></i>
                    }
                    <span class="font-semibold">(article.author.username.clone())</span>
                </a>
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

            <a href=(href.clone())>
                <h2 class="text-2xl font-bold mb-2 text-gray-800 dark:text-gray-200">
                    (article.title.clone())
                </h2>
            </a>
            <a href=(href.clone())>
                <p class="text-gray-700 dark:text-gray-300 mb-4">
                    (article.description.clone())
                </p>
            </a>

            <div
                class="flex flex-wrap justify-between items-end gap-2 text-gray-600 dark:text-gray-400"
            >
                <a href=(href) class="hover:text-blue-600 hover:underline">
                    "Read more..."
                </a>
                <div class="flex flex-wrap gap-1 items-center">
                    <i class="fa-solid fa-hashtag py-1"></i>
                    for t in article.tag_list {
                        if !t.is_empty() {
                            <a
                                href=(format!("/?page=0&amount=10&tag={t}&my_feed=false"))
                                class="bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 px-2 py-1 rounded text-xs hover:bg-green-200 dark:hover:bg-green-800"
                            >
                                (t)
                            </a>
                        }
                    }
                </div>
            </div>
        </div>
    })
}
