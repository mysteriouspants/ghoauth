use crate::{error::Error, shapes::GetAccessTokenResponse, UserDetailResponse};

use reqwest::Client as ReqwestClient;

const BASE_URL: &str = "https://github.com";
const API_BASE_URL: &str = "https://api.github.com";

/// A client for interacting with Github programmatically.
#[derive(Clone)]
pub struct GithubClient {
    http_client: ReqwestClient,
    /// The github client id. This one gets exposed publicly.
    client_id: String,
    /// The secret key that is known only to us and Github. Keep this
    /// one private!
    client_secret: String,
    /// The base url that authorization urls are based on.
    base_url: &'static str,
    /// The base url that the client uses to communicate with Github.
    api_base_url: &'static str,
}

impl GithubClient {
    /// Create a new Github client configured to use the public Github
    /// API.
    pub fn new(client_id: &str, client_secret: &str) -> Result<Self, Error> {
        Self::new_with_urls(client_id, client_secret, BASE_URL, API_BASE_URL)
    }

    /// Create a new Github client configured to use arbitrary API
    /// endpoints.
    ///
    /// See also [`crate::fakehub::Fakehub::add_client`].
    pub fn new_with_urls(
        client_id: &str,
        client_secret: &str,
        base_url: &'static str,
        api_base_url: &'static str,
    ) -> Result<Self, Error> {
        Ok(Self {
            http_client: reqwest::ClientBuilder::new()
                .user_agent("Rust/request/ghoauth")
                .build()?,
            client_id: client_id.to_owned(),
            client_secret: client_secret.to_owned(),
            base_url,
            api_base_url,
        })
    }

    /// The URL to send a user to in order to start the OAuth workflow.
    pub fn authorization_url(&self) -> String {
        format!(
            "{}/login/oauth/authorize?client_id={}",
            self.base_url, self.client_id
        )
    }

    /// Exchange a login code for an access token.
    pub async fn get_access_token(&self, code: &str) -> Result<GetAccessTokenResponse, Error> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
        ];

        Ok(self
            .http_client
            .post(format!("{}/login/oauth/access_token", self.base_url))
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await?
            .json()
            .await?)
    }

    /// Use an access token to query the user this token is associated with.
    pub async fn get_user_detail(&self, access_token: &str) -> Result<UserDetailResponse, Error> {
        Ok(self
            .http_client
            .get(format!("{}/user", self.api_base_url))
            .header("Authorization", format!("token {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await?
            .json()
            .await?)
    }
}

impl std::fmt::Debug for GithubClient {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "GithubClient {{ http_client: {:?}, client_id: {}, \
            client_secret: REDACTED }}",
            self.http_client, self.client_id,
        )
    }
}
