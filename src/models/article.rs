use crate::database::db;
use serde::{Deserialize, Serialize};
use topcoat::context::Cx;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPreview {
    pub username: String,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Article {
    pub slug: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub description: String,
    pub created_at: String,
    pub favorites_count: i64,
    pub comments_count: i64,
    pub tag_list: Vec<String>,
    pub author: UserPreview,
    pub fav: bool,
}

impl Article {
    pub async fn load_profile_articles(
        cx: &Cx,
        username: &str,
        favourites: bool,
        page: i64,
        amount: i64,
        user_id: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = page * amount;
        sqlx::query!(
                "
    SELECT
        a.slug,
        a.title,
        a.description,
        a.created_at,
        u.username,
        u.image,
        (SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
        (SELECT COUNT(*) FROM comments WHERE article=a.slug) as comments_count,
        EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and user_id=$2) as fav,
        EXISTS(SELECT 1 FROM Follows WHERE follower_id=$2 and influencer_id=u.id) as following,
        (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
    FROM Articles as a
        JOIN Users as u ON u.id = a.author_id
    WHERE
        CASE WHEN $3 THEN
            EXISTS(SELECT fa.article, fa.user_id FROM FavArticles as fa WHERE fa.article=a.slug AND fa.user_id=(SELECT id FROM Users WHERE username=$1))
        ELSE u.username = $1
        END
        ORDER BY a.created_at desc
        LIMIT $4 OFFSET $5",
                username,
                user_id,
                favourites,
                amount,
                offset,
            )
            .map(|x| Self {
                slug: x.slug,
                title: x.title,
                body: None, // no need
                fav: x.fav != 0,
                description: x.description,
                created_at: x.created_at.format(crate::models::DATE_FORMAT).to_string(),
                favorites_count: x.favorites_count,
                tag_list: x
                    .tag_list
                    .map(|x| x.split(' ').map(ToString::to_string).collect::<Vec<_>>())
                    .unwrap_or_default(),
                author: UserPreview {
                    username: x.username,
                    image: x.image,
                    following: x.following !=0,
                },
                comments_count: x.comments_count,
            })
            .fetch_all(db(cx))
            .await
    }

    pub async fn fetch_article(
        cx: &Cx,
        slug: &str,
        user_id: Option<i64>,
    ) -> Result<Article, sqlx::Error> {
        sqlx::query!(
            r#"
    SELECT
        a.slug as slug,
        a.title as title,
        a.body as body,
        a.description as description,
        a.created_at as created_at,
        (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as "tag_list: Option<String>",
        (SELECT COUNT(*) FROM FavArticles WHERE article = a.slug) as "fav_count: Option<i64>",
        (SELECT COUNT(*) FROM comments WHERE article = a.slug) as "comments_count: Option<i64>",
        u.username as username,
        u.image as image,
        EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and user_id=$2) as "fav: Option<i64>",
        EXISTS(SELECT 1 FROM Follows WHERE follower_id=$2 and influencer_id=u.id) as "following: Option<i64>"
    FROM Articles a
        JOIN Users u ON a.author_id = u.id
    WHERE slug = $1
    "#,
            slug,
            user_id,
        )
        .map(|x| Article
            {
            slug: x.slug,
            title: x.title,
            description: x.description,
            body: Some(x.body),
            tag_list: x
                .tag_list
                .flatten()
                .as_deref()
                .unwrap_or_default()
                .split_ascii_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>(),
            favorites_count: x.fav_count.flatten().unwrap_or_default(),
            created_at: x.created_at.format(super::DATE_FORMAT).to_string(),
            fav: x.fav.flatten().unwrap_or_default() != 0,
            author: UserPreview {
                username: x.username,
                image: x.image,
                following: x.following.flatten().unwrap_or_default() != 0,
            },
            comments_count: x.comments_count.flatten().unwrap_or_default(),
        })
        .fetch_one(db(cx))
        .await
    }

    pub async fn load_home_articles(
        cx: &Cx,
        page: i64,
        amount: i64,
        tag: &str,
        my_feed: bool,
        user_id: Option<i64>,
    ) -> Result<Vec<Article>, sqlx::Error> {
        let offset = page * amount;
        // let username = username.unwrap_or("");

        sqlx::query!(
            "
SELECT
a.slug,
a.title,
a.description,
a.created_at,
(SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
(SELECT COUNT(*) FROM comments WHERE article=a.slug) as comments_count,
u.username, u.image,
EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and user_id=$5) as fav,
EXISTS(SELECT 1 FROM Follows WHERE follower_id=$5 and influencer_id=u.id) as following,
(SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
FROM Articles as a
JOIN Users as u ON a.author_id = u.id
WHERE
CASE WHEN $3!='' THEN a.slug in (SELECT distinct article FROM ArticleTags WHERE tag=$3)
ELSE 1=1
END
AND
CASE WHEN $4 THEN u.id in (SELECT influencer_id FROM Follows WHERE follower_id=$5)
ELSE 1=1
END
ORDER BY a.created_at desc
LIMIT $1 OFFSET $2",
            amount,
            offset,
            tag,
            my_feed,
            user_id,
        )
        .map(|x| Article {
            slug: x.slug,
            title: x.title,
            body: None, // no need
            fav: x.fav != 0,
            // fav: x.fav.unwrap_or_default(),
            description: x.description,
            created_at: x.created_at.format(crate::models::DATE_FORMAT).to_string(),
            favorites_count: x.favorites_count,
            // favorites_count: x.favorites_count.unwrap_or_default(),
            author: UserPreview {
                username: x.username,
                image: x.image,
                following: x.following != 0,
                // following: x.following.unwrap_or_default(),
            },
            tag_list: x
                .tag_list
                .unwrap_or_default()
                .split(' ')
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
            comments_count: x.comments_count,
            // comments_count: x.comments_count.unwrap_or_default(),
        })
        .fetch_all(db(cx))
        .await
    }
}
