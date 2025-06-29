#![forbid(unsafe_code)]

use anyhow::Context;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Form, Router};
use clap::Parser;
use maud::{DOCTYPE, html};
use serde::Deserialize;
use sqlx::{Acquire, Sqlite};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::instrument;

macro_rules! layout {
    ($content:expr) => {
        html! {
            (DOCTYPE)
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title {
                    "journal"
                }
                script
                    src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js"
                    integrity="sha384-Akqfrbj/HpNVo8k11SXBb6TlBWmXXlYQrCSqEWmyKJe+hDm3Z/B2WVG4smwBkRVm"
                    crossorigin="anonymous" {}
                link
                    rel="stylesheet"
                    href="https://cdn.jsdelivr.net/npm/bulma@1.0.4/css/bulma.min.css";
                style {
                    "
                    pre {
                        white-space: pre-wrap;
                    }
                    "
                }
            }
            body {
                ($content)
            }
        }
    };
}

fn main_template(flash: Option<maud::Markup>) -> maud::Markup {
    html! {
        section
            id="main"
            class="section"
        {
            div
                class="container"
            {
                form
                    id="entry-input-form"
                    hx-post="/entry"
                    hx-target="#main"
                    hx-swap="outerHTML"
                {
                    div class="field" {
                        div class="control" {
                            textarea
                                class="textarea"
                                name="body"
                                rows="10" {}
                        }
                    }

                    @if let Some(flash) = flash {
                        (flash)
                    }

                    div class="field-body" {
                        div class="field is-grouped" {
                            div class="control" {
                                button
                                    type="submit"
                                    class="button is-link"
                                {
                                    "Submit"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[instrument]
async fn journal_index() -> axum::response::Result<impl IntoResponse> {
    Ok(layout! {
        (main_template(None))
    })
}

#[derive(Deserialize, Debug)]
struct NewEntry {
    body: String,
}

#[instrument]
async fn create_entry(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(new_entry): Form<NewEntry>,
) -> axum::response::Result<impl IntoResponse> {
    let state = state.lock().await;

    let mut conn = state.pool.acquire().await.map_err(|e| {
        e.to_string();
    })?;

    let insert_result: Result<(i64,), sqlx::Error> = sqlx::query_as(
        "insert into entries (body) values (?)
        returning length(body)",
    )
    .bind(new_entry.body.trim())
    .fetch_one(&mut *conn)
    .await;

    let flash = match insert_result {
        Ok((character_count,)) => html! {
            div class="field" {
                p
                    class="help is-success"
                    hx-get="/empty"
                    hx-trigger="load delay:5s"
                    hx-target="this"
                    hx-swap="outerHTML"
                {
                    (format!("Created. Length: {character_count} chars."))
                }
            }
        },
        Err(e) => {
            html! {
                div class="field" {
                    p
                        class="help is-danger"
                        hx-get="/empty"
                        hx-trigger="load delay:5s"
                        hx-target="this"
                        hx-swap="outerHTML"
                    {
                        (e.to_string())
                    }
                }
            }
        }
    };

    Ok(main_template(Some(flash)))
}

#[derive(Debug, Parser)]
struct Options {
    #[arg(long, env = "JOURNAL_DB_PATH")]
    database_path: Option<PathBuf>,
    #[arg(long, env, default_value = "9999")]
    port: u16,
}

#[derive(Debug)]
struct AppState {
    pool: sqlx::Pool<Sqlite>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let options = Options::parse();

    let database_path = if let Some(database_path) = options.database_path {
        database_path
    } else {
        let mut database_path = directories::ProjectDirs::from("", "", "journal")
            .expect("must be able to find home directory")
            .data_local_dir()
            .to_path_buf();

        std::fs::create_dir_all(&database_path)?;
        database_path.push("journal.db");
        database_path
    };

    let db_opts = sqlx::sqlite::SqliteConnectOptions::from_str(&format!(
        "sqlite://{}",
        database_path
            .to_str()
            .context("could not turn database path to string")?
    ))?
    .busy_timeout(std::time::Duration::from_secs(5))
    .journal_mode(sqlx::sqlite::SqliteJournalMode::Delete)
    .create_if_missing(true)
    .foreign_keys(true);

    let pool = sqlx::SqlitePool::connect_with(db_opts).await?;

    let mut connection = pool.acquire().await?;

    let mut txn = connection.begin().await?;

    sqlx::query(
        "create table if not exists entries (
            id integer primary key autoincrement not null,
            body text not null check(length(trim(body)) > 0),
            inserted_at datetime not null default(STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
            updated_at datetime not null default(STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW'))
        )",
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query("create index if not exists entries_inserted_at on entries (inserted_at);")
        .execute(&mut *txn)
        .await?;

    sqlx::query(
        "create trigger if not exists entries_updated_at after update on entries
        begin
            update entries set updated_at = STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')
            where id = old.id;
        end;",
    )
    .execute(&mut *txn)
    .await?;

    txn.commit().await?;

    let state = Arc::new(Mutex::new(AppState { pool }));

    let app = Router::new()
        .route("/", get(journal_index))
        .route("/entry", post(create_entry))
        .route("/empty", get(|| async { "" }))
        .route(
            "/dev/state",
            get(|State(state): State<Arc<Mutex<AppState>>>| async move {
                let state = state.lock().await;
                format!("{state:#?}")
            }),
        )
        .with_state(state)
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", options.port)).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
