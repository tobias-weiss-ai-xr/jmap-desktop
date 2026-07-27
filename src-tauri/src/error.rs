use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("JMAP connection error: {0}")]
    Connection(#[from] reqwest::Error),

    #[error("JMAP API error {status}: {detail}")]
    Api { status: u16, detail: String },

    #[error("Not connected — no active JMAP session")]
    NotConnected,

    #[error("Invalid server URL: {0}")]
    InvalidUrl(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("JMAP method error: {method} — {description}")]
    Method { method: String, description: String },

    #[error("{0}")]
    Other(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("type", &self.to_string())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
