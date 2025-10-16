use std::{collections::BTreeMap, sync::OnceLock};

use askama::Template;
use askama_web::WebTemplate;
use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use tower_http::services::ServeDir;

static MEME_COUNTS: OnceLock<BTreeMap<u32, usize>> = OnceLock::new();

#[derive(Template, WebTemplate)]
#[template(path = "home.html")]
struct HomeTemplate {}

async fn home() -> impl IntoResponse {
    HomeTemplate {}
}

#[derive(Template, WebTemplate)]
#[template(path = "meme.html")]
struct MemeTemplate {
    img: String,
    prev: String,
    next: String,
}

async fn meme(Path((cid, id)): Path<(u32, u32)>) -> impl IntoResponse {
    let total = MEME_COUNTS.get().unwrap().get(&cid).unwrap();
    let dir = std::fs::read_dir(format!("static/collections/{cid}/memes")).unwrap();
    let mut file = None;
    for f in dir {
        let f = f.unwrap().file_name().to_str().unwrap().to_owned();
        if [
            format!("{id}.jpg"),
            format!("{id}.jpeg"),
            format!("{id}.png"),
            format!("{id}.gif"),
        ]
        .contains(&f)
        {
            file = Some(f);
            break;
        }
    }
    let file = file.unwrap_or(format!("{id}.jpg"));

    MemeTemplate {
        img: format!("/static/collections/{cid}/memes/{file}"),
        prev: if id - 1 > 0 {
            format!("{}", id - 1)
        } else {
            "".into()
        },
        next: if id < *total as u32 {
            format!("{}", id + 1)
        } else {
            "".into()
        },
    }
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    let mut btm = BTreeMap::new();
    for col in std::fs::read_dir("static/collections").unwrap() {
        let col = col.unwrap();
        btm.insert(
            col.file_name().to_str().unwrap().parse().unwrap(),
            std::fs::read_dir(col.path().join("memes")).unwrap().count(),
        );
    }
    MEME_COUNTS.set(btm).unwrap();

    let router = Router::new()
        .route("/", get(home))
        .route("/collections/{cid}/memes/{id}", get(meme))
        .nest_service("/static", ServeDir::new("static"));

    Ok(router.into())
}
