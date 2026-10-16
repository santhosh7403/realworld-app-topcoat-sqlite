use topcoat::{
    Result,
    view::{View, component, view},
};

#[component]
pub async fn prev_next(
    page: i64,
    amount: i64,
    tag: String,
    my_feed: bool,
    article_count: i64,
) -> Result<impl View> {
    let prev_href = format!(
        "/?page={}&amount={}&tag={}&my_feed={}",
        (page - 1).max(0),
        amount,
        tag,
        my_feed
    );
    let next_href = format!(
        "/?page={}&amount={}&tag={}&my_feed={}",
        page + 1,
        amount,
        tag,
        my_feed
    );
    let btn = "px-4 cursor-pointer hover:text-blue-500 border dark:border-gray-600 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200";

    Ok(view! {
        if page > 0 {
            <a href=(prev_href) class=(btn)>"<< Previous page"</a>
        }
        if article_count > 0 && article_count >= amount {
            <a href=(next_href) class=(btn)>"Next page >>"</a>
        }
    })
}

#[component]
pub async fn prev_next_profile(
    page: i64,
    amount: i64,
    favourites: bool,
    article_count: i64,
    username: &str,
) -> Result<impl View> {
    let prev_href = format!(
        "/profile/{}/?page={}&amount={}&favourites={}",
        username,
        (page - 1).max(0),
        amount,
        favourites
    );
    let next_href = format!(
        "/profile/{}/?page={}&amount={}&favourites={}",
        username,
        page + 1,
        amount,
        favourites
    );
    let btn = "px-4 cursor-pointer hover:text-blue-500 border dark:border-gray-600 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200";

    Ok(view! {
        if page > 0 {
            <a href=(prev_href) class=(btn)>"<< Previous page"</a>
        }
        if article_count > 0 && article_count >= amount {
            <a href=(next_href) class=(btn)>"Next page >>"</a>
        }
    })
}
