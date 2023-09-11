use std::{net::SocketAddr, sync::Arc};

use tokio::sync::Mutex;
use url::Url;

use crate::fakehub::state::UserId;
use crate::GithubClient;

use super::{
    api_gh::ApiDotGithubDotCom,
    error::Result,
    gh::GithubDotCom,
    state::{Client, FakehubState, FakehubStateRef, User},
};

/// A fake implementation of github.com and api.github.com, complete
/// enough for use in an integration test.
pub struct Fakehub {
    state: FakehubStateRef,
    root_server: GithubDotCom,
    api_server: ApiDotGithubDotCom,
}

impl Fakehub {
    /// Create a new Fakehub.
    pub fn new() -> Result<Self> {
        let state = Arc::new(Mutex::new(FakehubState::new()));

        let root_server = GithubDotCom::new(3050, state.clone())?;
        let api_server = ApiDotGithubDotCom::new(3051, state.clone())?;

        Ok(Self {
            root_server,
            api_server,
            state,
        })
    }

    /// Add a Client to this Fakehub instance and return a GithubClient configured to use it.
    pub async fn add_client(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<GithubClient, crate::Error> {
        let mut state = self.state.lock().await;

        state.clients.insert(
            client_id.to_owned(),
            Client {
                secret: client_secret.to_owned(),
                redirect_url: Url::parse("http://127.0.0.1").unwrap(),
            },
        );

        GithubClient::new_with_urls(
            client_id,
            client_secret,
            self.github_dot_com_url().leak(),
            self.api_dot_github_dot_com_url().leak(),
        )
    }

    /// Add a User to this Fakehub instance.
    pub async fn add_user(&self, user_id: i64, user: User) {
        let mut state = self.state.lock().await;

        state.users.insert(user_id, user);
    }

    /// Simulate a user going to your authorization URL to log in, which
    /// returns a simple code. The code must not be confused with an API
    /// token, which the backend service (not the user) exchanges the code
    /// for.
    pub async fn get_code(&self, user_id: UserId) -> Result<String> {
        let mut state = self.state.lock().await;
        let user_id = user_id.to_owned();

        state.get_code(user_id)
    }

    /// Shutdown this fakehub.
    pub async fn shutdown(self) {
        self.root_server.shutdown().await;
        self.api_server.shutdown().await;
    }

    pub fn github_dot_com_url(&self) -> String {
        format!(
            "http://{}:{}",
            self.github_dot_com_socket().ip(),
            self.github_dot_com_socket().port()
        )
    }

    pub fn github_dot_com_socket(&self) -> &SocketAddr {
        &self.root_server._temp_server.socket_addr
    }

    pub fn api_dot_github_dot_com_url(&self) -> String {
        format!(
            "http://{}:{}",
            self.api_dot_github_dot_com_socket().ip(),
            self.api_dot_github_dot_com_socket().port()
        )
    }

    pub fn api_dot_github_dot_com_socket(&self) -> &SocketAddr {
        &self.api_server._temp_server.socket_addr
    }
}
