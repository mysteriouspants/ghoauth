use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use url::Url;

use super::{Error, Result};

pub(crate) type FakehubStateRef = Arc<Mutex<FakehubState>>;
pub(crate) type ClientId = String;
pub(crate) type Code = String;
pub(crate) type UserId = i64;
pub(crate) type Token = String;

#[derive(Debug)]
pub struct Client {
    pub secret: String,
    pub redirect_url: Url,
}

#[derive(Debug)]
pub struct User {
    pub login: String,
    pub avatar_url: String,
    pub html_url: String,
}

#[derive(Debug)]
pub struct FakehubState {
    pub users: HashMap<UserId, User>,
    pub clients: HashMap<ClientId, Client>,
    pub issued_codes: HashMap<Code, UserId>,
    pub tokens: HashMap<Token, UserId>,
}

impl FakehubState {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            clients: HashMap::new(),
            issued_codes: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    pub fn get_client(&self, client_id: &str) -> Result<&Client> {
        match self.clients.get(client_id) {
            Some(client) => Ok(client),
            None => Err(Error::NoSuchClient(client_id.to_owned())),
        }
    }

    /// Get a login code for a given user id.
    pub fn get_code(&mut self, user_id: UserId) -> Result<String> {
        let user_id = user_id.to_owned();

        if !self.users.contains_key(&user_id) {
            return Err(Error::NoSuchUserId(user_id));
        }

        let issued_code = format!("token_{}", user_id);

        self.issued_codes.insert(issued_code.clone(), user_id);

        Ok(issued_code)
    }

    /// Whether a given client/secret matches the list of known clients.
    pub fn client_matches(&self, client_id: &str, client_secret: &str) -> bool {
        self.clients
            .get(client_id)
            .map(|client| client.secret == client_secret)
            .unwrap_or(false)
    }

    /// Gets a code out of the store, removing it. Prepares for calling
    /// push_token.
    pub fn pop_code(&mut self, code: &str) -> Option<UserId> {
        self.issued_codes.remove(code)
    }

    pub fn push_token(&mut self, user_id: UserId) -> Token {
        let issued_token = format!("token_{}", user_id);

        self.tokens.insert(issued_token.to_owned(), user_id);

        issued_token
    }

    pub fn get_user_by_login(&self, login: &str) -> Option<(&UserId, &User)> {
        self.users.iter().find(|u| u.1.login == login)
    }
}

impl Client {
    /// Approximates Github's rules for redirects. See
    /// https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#redirect-urls
    /// for the canonical reference. This method picks the supplied redirect URL if it is more
    /// specific than the base URL.
    pub fn check_redirect_url(&self, redirect_url: &Url) -> Result<Url> {
        match self.redirect_url.host_str() {
            Some(base_host) => {
                if let Some(redirect_host) = redirect_url.host_str() {
                    if redirect_host != base_host {
                        return Err(Error::InvalidHost(
                            base_host.to_owned(),
                            redirect_host.to_owned(),
                        ));
                    }
                }
            }
            None => return Err(Error::HostlessBase(format!("{}", self.redirect_url))),
        }

        if !redirect_url.path().starts_with(self.redirect_url.path()) {
            return Err(Error::InvalidBasePath(
                self.redirect_url.path().to_owned(),
                redirect_url.path().to_owned(),
            ));
        }

        Ok(redirect_url.clone())
    }
}
