use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetAccessTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

// Custom debug printer omits the access token, which should never be
// logged for security reasons.
impl std::fmt::Debug for GetAccessTokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "GetAccessTokenResponse {{ access_token: REDACTED, token_type: \
                {}, scope: {} }}",
            self.token_type, self.scope,
        )
    }
}

/// The structure we map the user details from Github onto for an
/// internal user record.
///
/// Broadly speaking, these are the only fields we're truly interested
/// in from Github. The id is the most important, for it is how we can
/// durably refer to a user even if they change their alias on Github.
/// The login pre-populates a user's identity, and the avatar and link
/// to their github might become useful in the future, though it's not a
/// sure thing.
#[derive(Deserialize, Debug, Serialize)]
pub struct UserDetailResponse {
    pub id: i64,
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
}
