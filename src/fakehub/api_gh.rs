use axum::extract::Path;
use axum::{extract::State, http::HeaderMap, routing::get, Json, Router};

use crate::{
    fakehub::{error::Error, state::FakehubStateRef},
    UserDetailResponse,
};

use super::{error::Result, temp_server::TempServer};

/// A fake implementation of api.github.com, complete enough to stand in
/// for the real thing in an integration tested OAuth flow. Which isn't
/// very complete at all, now is it?
pub struct ApiDotGithubDotCom {
    pub _temp_server: TempServer,
}

impl ApiDotGithubDotCom {
    /// Create and start a new fake api.github.com.
    pub fn new(starting_port: u16, fakehub_state: FakehubStateRef) -> Result<Self> {
        let app = Router::new()
            .route("/user", get(get_user_detail))
            .route("/user/:login", get(get_user_detail_public))
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

async fn get_user_detail(
    State(fakehub_state): State<FakehubStateRef>,
    headers: HeaderMap,
) -> Result<Json<UserDetailResponse>> {
    let authorization = match headers.get("Authorization") {
        Some(authorization) => match authorization.to_str() {
            Ok(authorization) => authorization,
            Err(_) => return Err(Error::InvalidHeader("Authorization".to_string())),
        },
        None => return Err(Error::Unauthorized),
    };
    let authorization = match authorization.strip_prefix("token ") {
        Some(authorization) => authorization.to_string(),
        None => return Err(Error::Unauthorized),
    };
    let fakehub_state = fakehub_state.lock().await;
    let user_id = match fakehub_state.tokens.get(&authorization) {
        Some(user_id) => user_id,
        None => return Err(Error::Unauthorized),
    };
    let user = match fakehub_state.users.get(user_id) {
        Some(user) => user,
        None => return Err(Error::NoSuchUserId(*user_id)),
    };

    Ok(Json(UserDetailResponse {
        id: *user_id,
        login: user.login.clone(),
        html_url: user.html_url.clone(),
        avatar_url: user.avatar_url.clone(),
    }))
}

async fn get_user_detail_public(
    State(fakehub_state): State<FakehubStateRef>,
    Path(login): Path<String>,
) -> Result<Json<UserDetailResponse>> {
    let fakehub_state = fakehub_state.lock().await;
    let user = match fakehub_state.get_user_by_login(&login) {
        Some(user) => user,
        None => return Err(Error::NoSuchUserLogin(login)),
    };

    Ok(Json(UserDetailResponse {
        id: *user.0,
        login: user.1.login.clone(),
        html_url: user.1.html_url.clone(),
        avatar_url: user.1.avatar_url.clone(),
    }))
}
