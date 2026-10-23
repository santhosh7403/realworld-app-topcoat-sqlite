use crate::database::db;
use serde::{Deserialize, Serialize};
use topcoat::context::Cx;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchedArticles {
    pub slug: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
}

impl MatchedArticles {
    pub async fn search_count(cx: &Cx, query: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query!(
            r#"SELECT COUNT(*) as "tot: i64" FROM Articles_fts AS AFTS JOIN Articles AS A ON A.oid = AFTS.rowid WHERE Articles_fts MATCH $1"#,
            query
        )
        .fetch_one(db(cx))
        .await?;
        Ok(row.tot)
    }

    pub async fn search_articles(
        cx: &Cx,
        query: &str,
        page: i64,
        amount: i64,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let offset = page * amount;
        sqlx::query!(
            r#"
SELECT distinct
a.slug as slug,
snippet(articles_fts,1, '<span class="bg-yellow-300 text-gray-700">','</span>','<span class="bg-yellow-300 text-gray-700">  ...  </span>',10) as "title: String",
snippet(articles_fts,2, '<span class="bg-yellow-300 text-gray-700">','</span>','<span class="bg-yellow-300 text-gray-700">  ...  </span>',20) as "description: String",
snippet(articles_fts,3, '<span class="bg-yellow-300 text-gray-700">','</span>','<span class="bg-yellow-300 text-gray-700">  ...  </span>',20) as "body: String"
FROM Articles_fts AS AFTS
JOIN  Articles AS A  ON A.oid = AFTS.rowid
WHERE Articles_fts MATCH $3
order by rank
LIMIT $1 OFFSET $2"#,
            amount,
            offset,
            query,
        )
        .map(|x| Self {
            slug: x.slug,
            title: x.title,
            description: x.description,
            body: x.body,
        })
        .fetch_all(db(cx))
        .await
    }
}
