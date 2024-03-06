use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use url::Url;

use super::{error::Result, state::FakehubStateRef, temp_server::TempServer};

/// A fake implementation of github.com, complete enough to stand in for
/// the real thing in an integration tested OAuth flow. Which isn't very
/// complete at all, now is it?
#[derive(Debug)]
pub struct GithubDotCom {
    pub _temp_server: TempServer,
}

impl GithubDotCom {
    /// Create and start a new fake github.com.
    pub fn new(starting_port: u16, fakehub_state: FakehubStateRef) -> Result<Self> {
        let app = Router::new()
            .route("/", get(root))
            .route("/login/oauth/authorize", get(login_page).post(issue_code))
            .route("/login/oauth/access_token", post(exchange_code_for_token))
            .with_state(fakehub_state);

        Ok(Self {
            _temp_server: TempServer::new(starting_port, app)?,
        })
    }

    /// End it all.
    pub async fn shutdown(self) {
        self._temp_server.shutdown().await;
    }
}

// GET /
async fn root() -> &'static str {
    "Hello, world!"
}

// GET /login/oauth/authorize?client_id=:client_id
async fn login_page(
    State(fakehub_state): State<FakehubStateRef>,
    Query(LoginPageQueryParams { client_id }): Query<LoginPageQueryParams>,
) -> String {
    let fakehub_state = fakehub_state.lock().await;

    super::login_page::render(
        &client_id,
        fakehub_state
            .users
            .iter()
            .map(|(k, v)| (*k, v.login.as_str())),
    )
}

#[derive(Debug, Deserialize)]
struct LoginPageQueryParams {
    client_id: String,
}

// POST /login/oauth/authorize?client_id=:client_id&user_id=:user_id
// Expect redirect to application redirect url with the code
// this is only used in interactive test situations; in integration
// tests this is called directly through Fakehub
async fn issue_code(
    State(fakehub_state): State<FakehubStateRef>,
    Query(IssueCodeQueryParams {
        client_id,
        user_id,
        redirect_uri,
    }): Query<IssueCodeQueryParams>,
) -> Result<Response> {
    let redirect_uri = match redirect_uri {
        Some(redirect_uri) => Some(Url::parse(&redirect_uri)?),
        None => None,
    };
    let mut fakehub_state = fakehub_state.lock().await;
    let client = fakehub_state.get_client(&client_id)?;
    let mut redirect_uri = match redirect_uri {
        Some(redirect_uri) => client.check_redirect_url(&redirect_uri)?,
        None => client.redirect_url.clone(),
    };
    let code = fakehub_state.get_code(user_id)?;

    // check_redirect also picks the request-supplied redirect if it is
    // more specific, so all that remains is to add the issued code to
    // the uri's query parameters
    redirect_uri.query_pairs_mut().append_pair("code", &code);

    Ok((StatusCode::FOUND, [("Location", redirect_uri.as_str())]).into_response())
}

#[derive(Debug, Deserialize)]
struct IssueCodeQueryParams {
    client_id: String,
    user_id: i64,
    redirect_uri: Option<String>,
}

// POST /login/oauth/access_token
async fn exchange_code_for_token(
    State(fakehub_state): State<FakehubStateRef>,
    Form(ExchangeCodeForTokenFormParams {
        client_id,
        client_secret,
        code,
    }): Form<ExchangeCodeForTokenFormParams>,
) -> ExchangeCodeForTokenResponse {
    let mut fakehub_state = fakehub_state.lock().await;

    if !fakehub_state.client_matches(&client_id, &client_secret) {
        return ExchangeCodeForTokenResponse::BadClient;
    }

    let user_id = match fakehub_state.pop_code(&code) {
        Some(code) => code,
        None => {
            return ExchangeCodeForTokenResponse::BadCode;
        }
    };

    let token = fakehub_state.push_token(user_id);

    ExchangeCodeForTokenResponse::Token(Json(TokenResponse {
        access_token: token.to_owned(),
        scope: "".to_owned(),
        token_type: "bearer".to_owned(),
    }))
}

#[derive(Debug, Deserialize)]
struct ExchangeCodeForTokenFormParams {
    client_id: String,
    client_secret: String,
    code: String,
}

enum ExchangeCodeForTokenResponse {
    Token(Json<TokenResponse>),
    BadClient,
    BadCode,
}

impl IntoResponse for ExchangeCodeForTokenResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Token(t) => t.into_response(),
            Self::BadClient => (StatusCode::FORBIDDEN, "Forbidden - Bad Client").into_response(),
            Self::BadCode => (StatusCode::FORBIDDEN, "Forbidden - Bad Code").into_response(),
        }
    }
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    scope: String,
    token_type: String,
}
