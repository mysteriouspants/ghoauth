//! Super-simple client for interacting with the Github API. This is
//! incomplete, with only enough API calls to get through an OAuth
//! workflow.
//!
//! This also includes a "fakehub" that can be used for automated
//! testing by mocking out the Github website and API.
//!
//! ```
//! # use ghoauth::{GithubClient, fakehub::{Error, Fakehub, User}};
//! # const CLIENT_ID: &str = "1234567890";
//! # const CLIENT_SECRET: &str = "SECRET_SQUIRREL_STUFF";
//! # const USER: &str = "user";
//! # const USER_ID: i64 = 1;
//! # const USER_AVATAR_URL: &str = "https://github.com/";
//! # const USER_HTML_URL: &str = "https://github.com/";
//! # #[tokio::main]
//! # async fn main() -> Result<(), Error> {
//! // Fakehub is a mock Github with just enough functionality to drive
//! // a login flow.
//! let fakehub = Fakehub::new()
//!     .expect("cannot start local fakehub server");
//! // Train Fakehub with some fake data with add_client and add_user.
//! // Outside of tests, use GithubClient::new.
//! let github_client = fakehub.add_client(CLIENT_ID, CLIENT_SECRET)
//!     .await?;
//! // train fakehub with a pre-configured user
//! fakehub.add_user(USER_ID, User {
//!     login: USER.to_string(),
//!     avatar_url: USER_AVATAR_URL.to_string(),
//!     html_url: USER_HTML_URL.to_string()
//! }).await;
//! // Simulate a user visiting github's login page with your oauth
//! // application, logging in, and being redirected to your application
//! // with a code.
//! let code = fakehub.get_code(USER_ID).await?;
//! // Exchange that code for an api token.
//! let token = github_client.get_access_token(&code).await?;
//! // Ask github about the user that just authenticated.
//! let user_detail = github_client.get_user_detail(
//!     &token.access_token
//! ).await?;
//!
//! assert_eq!(USER, user_detail.login);
//!
//! fakehub.shutdown().await;
//! # Ok(())
//! # }
//! ```

pub use crate::{
    client::GithubClient,
    error::Error,
    shapes::{GetAccessTokenResponse, UserDetailResponse},
};

mod client;
mod error;
mod shapes;

#[cfg(feature = "fakehub")]
pub mod fakehub;

#[cfg(test)]
mod tests {
    use crate::fakehub::{Fakehub, User};

    const CLIENT_ID: &str = "1234567890";
    const CLIENT_SECRET: &str = "SECRET_SQUIRREL_STUFF";
    const USER: &str = "user";
    const USER_ID: i64 = 1;
    const USER_AVATAR_URL: &str = "https://github.com/";
    const USER_HTML_URL: &str = "https://github.com/";

    #[tokio::test]
    async fn oauth_flow() {
        let fakehub = Fakehub::new().expect("cannot start local fakehub server");

        let github_client = fakehub.add_client(CLIENT_ID, CLIENT_SECRET).await.unwrap();
        fakehub
            .add_user(
                USER_ID,
                User {
                    login: USER.to_string(),
                    avatar_url: USER_AVATAR_URL.to_string(),
                    html_url: USER_HTML_URL.to_string(),
                },
            )
            .await;

        let code = fakehub.get_code(USER_ID).await.unwrap();
        let token = github_client.get_access_token(&code).await.unwrap();
        let user_detail = github_client
            .get_user_detail(&token.access_token)
            .await
            .unwrap();

        assert_eq!(USER, user_detail.login);

        fakehub.shutdown().await;
    }
}
