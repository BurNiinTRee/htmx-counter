use anyhow::{anyhow, Context};
use axum::{
    extract::{NestedPath, Query, State},
    response::{IntoResponse, Redirect, Response},
    Form, Router,
};
use axum_extra::routing::{RouterExt, TypedPath};
use axum_htmx::{HxBoosted, HxPushUrl};
use serde::{Deserialize, Serialize};
use sqlx::{query, SqlitePool};

use crate::{AppState, Result};
mod tmpl;

pub fn router() -> Router<AppState> {
    Router::new()
        .typed_get(get_counter)
        .typed_post(post_counter)
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
struct CounterState {
    count: i64,
}

#[derive(TypedPath)]
#[typed_path("/counter")]
pub struct Counter;

async fn get_counter(
    _: Counter,
    State(db): State<SqlitePool>,
    initial_value: Option<Query<CounterState>>,
) -> Result<tmpl::Counter> {
    let count = match initial_value {
        Some(Query(CounterState { count })) => count,
        None => default(&db).await?,
    };
    Ok(tmpl::Counter { count })
}

#[derive(Deserialize)]
struct CounterAction {
    count: i64,
    action: String,
}

fn concat_url(root: NestedPath, state: Option<CounterState>) -> String {
    match state {
        Some(state) => format!("{}{}", root.as_str(), Counter.with_query_params(state)),
        None => format!("{}{}", root.as_str(), Counter.to_uri()),
    }
}

async fn post_counter(
    _: Counter,
    State(db): State<SqlitePool>,
    root: NestedPath,
    HxBoosted(is_boosted): HxBoosted,
    Form(action): Form<CounterAction>,
) -> Result<Response> {
    Ok(match action.action.as_str() {
        "inc" => {
            let count = action.count + 1;
            let url = concat_url(root, Some(CounterState { count }));
            if is_boosted {
                (HxPushUrl(url.parse()?), tmpl::Count { count }).into_response()
            } else {
                Redirect::to(&url).into_response()
            }
        }
        "dec" => {
            let count = action.count - 1;
            let url = concat_url(root, Some(CounterState { count }));
            if is_boosted {
                (HxPushUrl(url.parse()?), tmpl::Count { count }).into_response()
            } else {
                Redirect::to(&url).into_response()
            }
        }
        "default" => {
            let url = concat_url(root, None);
            if is_boosted {
                (
                    HxPushUrl(url.parse()?),
                    tmpl::Count {
                        count: default(&db).await?,
                    },
                )
                    .into_response()
            } else {
                Redirect::to(&url).into_response()
            }
        }
        "set-default" => {
            query!(
                r#"UPDATE SettingsInt SET value = ? WHERE name = "DefaultCount""#,
                action.count
            )
            .execute(&db)
            .await?;
            let url = concat_url(root, None);
            if is_boosted {
                (
                    HxPushUrl(url.parse()?),
                    tmpl::Count {
                        count: action.count,
                    },
                )
                    .into_response()
            } else {
                Redirect::to(&url).into_response()
            }
        }
        action => Err(anyhow!("{} is not a valid action", action))?,
    })
}

async fn default(db: &SqlitePool) -> Result<i64> {
    Ok(
        query!(r#"SELECT value FROM SettingsInt WHERE name = "DefaultCount""#)
            .fetch_one(db)
            .await?
            .value
            .context("No `DefaultCount` in database")?,
    )
}
