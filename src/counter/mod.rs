use anyhow::{anyhow, Context};
use axum::{
    extract::{NestedPath, Query, State},
    response::Redirect,
    Form, Router,
};
use axum_extra::routing::{RouterExt, TypedPath};
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
    Form(action): Form<CounterAction>,
) -> Result<Redirect> {
    Ok(match action.action.as_str() {
        "inc" => Redirect::to(&concat_url(
            root,
            Some(CounterState {
                count: action.count + 1,
            }),
        )),
        "dec" => Redirect::to(&concat_url(
            root,
            Some(CounterState {
                count: action.count - 1,
            }),
        )),
        "default" => Redirect::to(&concat_url(root, None)),
        "set-default" => {
            query!(
                r#"UPDATE SettingsInt SET value = ? WHERE name = "DefaultCount""#,
                action.count
            )
            .execute(&db)
            .await?;
            Redirect::to(&concat_url(root, None))
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
