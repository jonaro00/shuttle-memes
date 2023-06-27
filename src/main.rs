use std::{path::PathBuf, sync::OnceLock};

use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;

static MEME_COUNT: OnceLock<usize> = OnceLock::new();

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {}

async fn home() -> impl IntoResponse {
    let home = HomeTemplate {};
    HtmlTemplate(home)
}

#[derive(Template)]
#[template(path = "meme.html")]
struct MemeTemplate {
    img: String,
    prev: String,
    next: String,
}

async fn meme(Path(id): Path<u32>) -> impl IntoResponse {
    let total = MEME_COUNT.get().unwrap();
    let meme = MemeTemplate {
        img: format!("/static/memes/{}.jpg", id),
        prev: if id - 1 > 0 {
            format!("/meme/{}", id - 1)
        } else {
            "".into()
        },
        next: if id + 1 <= *total as u32 {
            format!("/meme/{}", id + 1)
        } else {
            "".into()
        },
    };
    HtmlTemplate(meme)
}

#[shuttle_runtime::main]
async fn axum(
    #[shuttle_static_folder::StaticFolder] static_folder: PathBuf,
) -> shuttle_axum::ShuttleAxum {
    MEME_COUNT
        .set(
            std::fs::read_dir(&static_folder.join("memes"))
                .unwrap()
                .count(),
        )
        .unwrap();

    let router = Router::new()
        .route("/", get(home))
        .route("/meme/:id", get(meme))
        .nest_service("/static", ServeDir::new(static_folder));

    Ok(router.into())
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
