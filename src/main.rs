use anyhow::Context;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_static_web_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let api_router = Router::new().route("/hello", get(say_hello));
    let assets_path = std::env::current_dir().unwrap();

    let app = Router::new()
        .route("/", get(home))
        .route("/learn", get(learn_more))
        .nest("/api", api_router)
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        );

    // run it
    let port = std::env::var("PORT").unwrap().parse::<u16>().unwrap();
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("error while starting API server")?;

    Ok(())
}

async fn home() -> impl IntoResponse {
    let template = HomeTemplate {};
    HtmlTemplate(template)
}

async fn learn_more() -> impl IntoResponse {
    let template = LearnMoreTemplate {};
    HtmlTemplate(template)
}

async fn say_hello() -> &'static str {
    "Hello!"
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomeTemplate;

#[derive(Template)]
#[template(path = "pages/learn-more.html")]
struct LearnMoreTemplate;

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
