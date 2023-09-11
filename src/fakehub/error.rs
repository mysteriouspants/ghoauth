use axum::http::StatusCode;
use axum::response::IntoResponse;
use thiserror::Error;

use crate::fakehub::state::UserId;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No free port available, tried from {0} upward.")]
    NoAvailablePorts(u16),
    #[error("No user with id {0} exists.")]
    NoSuchUser(UserId),
    #[error("Failed to parse URL {0}")]
    UrlParse(String),
    #[error("Authentication URL is missing a client id")]
    AuthUrlMissingClientId,
    #[error("Not authorized.")]
    Unauthorized,
    #[error("No client with id {0} in Fakehub")]
    NoSuchClient(String),
    #[error("Base redirect url {0} has no host")]
    HostlessBase(String),
    #[error("Host {1} does not match base of {0}")]
    InvalidHost(String, String),
    #[error("Header {0} contains an non-ASCII value")]
    InvalidHeader(String),
    #[error("Path {1} does not match base of {0}")]
    InvalidBasePath(String, String),
    #[error("{0}")]
    ClientCreation(String),
    #[error("Remote responded with status code {0:?} with reason {1}")]
    Http(Option<u16>, String),
    #[error("{0}")]
    Decode(String),
    #[error("{0}")]
    OtherHttp(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Self::UrlParse(format!("{}", value))
    }
}

impl From<crate::Error> for Error {
    fn from(value: crate::Error) -> Self {
        match value {
            crate::Error::ClientCreation(reason) => Self::ClientCreation(reason),
            crate::Error::Http(code, reason) => Self::Http(code, reason),
            crate::Error::Decode(reason) => Self::Decode(reason),
            crate::Error::OtherHttp(reason) => Self::OtherHttp(reason),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::NoAvailablePorts(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self)).into_response()
            }
            Self::NoSuchUser(_) => (StatusCode::UNAUTHORIZED, format!("{}", self)).into_response(),
            Self::UrlParse(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("{}", self)).into_response()
            }
            Self::AuthUrlMissingClientId => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("{}", self)).into_response()
            }
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, format!("{}", self)).into_response(),
            Self::NoSuchClient(_) => {
                (StatusCode::UNAUTHORIZED, format!("{}", self)).into_response()
            }
            Self::InvalidHeader(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("{}", self)).into_response()
            }
            Self::InvalidHost(_, _) => (StatusCode::FORBIDDEN, format!("{}", self)).into_response(),
            Self::InvalidBasePath(_, _) => {
                (StatusCode::FORBIDDEN, format!("{}", self)).into_response()
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self)).into_response(),
        }
    }
}
