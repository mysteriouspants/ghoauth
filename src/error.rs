use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    ClientCreation(String),
    #[error("Remote responded with status code {0:?} with reason {1}")]
    Http(Option<u16>, String),
    #[error("{0}")]
    Decode(String),
    #[error("{0}")]
    OtherHttp(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_builder() {
            return Self::ClientCreation(err.to_string());
        }

        if err.is_decode() {
            return Self::Decode(err.to_string());
        }

        if err.is_status() {
            return Self::Http(
                err.status().map(|status_code| status_code.as_u16()),
                err.to_string(),
            );
        }

        Self::OtherHttp(err.to_string())
    }
}
