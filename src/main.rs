use axum::{
    extract::FromRef,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use sqlx::SqlitePool;

use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

mod counter;

#[derive(FromRef, Clone)]
struct AppState {
    pool: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Hello, world!");
    let pool = SqlitePool::connect(
        std::env::var("DATABASE_URL")
            .as_deref()
            .unwrap_or("sqlite:data.db?mode=rwc"),
    )
    .await?;
    sqlx::migrate!().run(&pool).await?;

    let counter_path = "/my/cool";
    let app =
        Router::new()
            .route(
                "/",
                get(move || async move {
                    Redirect::to(&format!("{}{}", counter_path, counter::Counter))
                }),
            )
            .nest(counter_path, counter::router())
            .nest_service("/assets/", ServeDir::new("assets/"))
            .with_state(AppState { pool })
            .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

type Result<T, E = ServerError> = std::result::Result<T, E>;

#[derive(Debug)]
struct ServerError(anyhow::Error);

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}
