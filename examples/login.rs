use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router, Server,
};
use clap::Parser;
use ghoauth::GithubClient;
use maud::{html, DOCTYPE};
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Debug, Parser)]
struct Args {
    /// The non-secret Client ID
    #[clap(short = 'i', long = "id")]
    client_id: String,
    /// The client secret.
    #[clap(short = 's', long = "secret")]
    client_secret: String,
    /// The port to bind to.
    #[clap(short = 'p', long = "port", default_value_t = 4000u16)]
    port: u16,
    /// The callback url, if different from `/callback`.
    #[clap(short = 'l', long = "location", default_value = "callback")]
    callback_location: String,
}

#[derive(Clone, Debug)]
struct AppState {
    github_client: Arc<RwLock<GithubClient>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let app_state = AppState {
        github_client: Arc::new(RwLock::new(
            GithubClient::new(&args.client_id, &args.client_secret).unwrap(),
        )),
    };
    let app = Router::new()
        .route("/", get(login_page))
        .route(
            &format!("/{}", args.callback_location),
            get(callback_handler),
        )
        .with_state(app_state);

    println!("Now serving on http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn login_page(State(app_state): State<AppState>) -> Html<String> {
    let github_client = app_state.github_client.read().await;
    let fragment = html! {
        (DOCTYPE)

        html {
            head {
                title { "Login with Github" }
            }
            body {
                h1 {
                    "Login with Github"
                }
                p {
                    a href=(github_client.authorization_url()) { "Login with Github" }
                }
            }
        }
    };

    Html(fragment.into_string())
}

#[derive(Debug, Deserialize)]
struct CallbackHandlerQueryParams {
    code: String,
}

async fn callback_handler(
    State(app_state): State<AppState>,
    Query(CallbackHandlerQueryParams { code }): Query<CallbackHandlerQueryParams>,
) -> Response {
    let github_client = app_state.github_client.read().await;
    let access_token = match github_client.get_access_token(&code).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::FORBIDDEN, format!("{}", e)).into_response(),
    };
    let user_detail = match github_client
        .get_user_detail(&access_token.access_token)
        .await
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::FORBIDDEN, format!("{}", e)).into_response(),
    };
    let fragment = html! {
        (DOCTYPE)

        html {
            head {
                title {
                    "User detail for "
                    (user_detail.login)
                }
            }
            body {
                h1 {
                    "User detail for "
                    (user_detail.login)
                }
                table {
                    thead {
                        tr {
                            th { "Field" }
                            th { "Value" }
                            th { "Explanation" }
                        }
                    }
                    tbody {
                        tr {
                            td { "id" }
                            td { (user_detail.id) }
                            td { "numeric user id, this doesn't change even when a user changes their name on Github"}
                        }
                        tr {
                            td { "login" }
                            td { (user_detail.login) }
                            td { "the user's public github alias" }
                        }
                        tr {
                            td { "avatar" }
                            td { (user_detail.avatar_url) }
                            td { "url for the user's public picture" }
                        }
                        tr {
                            td { "html" }
                            td { (user_detail.html_url) }
                            td { "url for the user's public github profile" }
                        }
                    }
                    p { a href=(github_client.authorization_url()) { "Login as another user" } }
                }
            }
        }
    };

    Html(fragment.into_string()).into_response()
}
