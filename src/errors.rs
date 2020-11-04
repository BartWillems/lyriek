use derive_more::Display;
use std::convert::From;

#[derive(Debug, Display)]
pub enum LyriekError {
    PlayerError(String),
    #[display(fmt = "Player not found")]
    PlayerNotFound,
    Communication(String),
}

impl From<mpris::DBusError> for LyriekError {
    fn from(error: mpris::DBusError) -> LyriekError {
        LyriekError::PlayerError(format!("D-BUS error: {}", error))
    }
}

impl From<mpris::FindingError> for LyriekError {
    fn from(error: mpris::FindingError) -> LyriekError {
        LyriekError::PlayerError(format!("Player lookup error: {}", error))
    }
}

impl<Msg> From<std::sync::mpsc::SendError<Msg>> for LyriekError {
    fn from(error: std::sync::mpsc::SendError<Msg>) -> LyriekError {
        LyriekError::Communication(format!("communication error: {}", error))
    }
}
