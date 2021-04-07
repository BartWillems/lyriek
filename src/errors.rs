use derive_more::Display;
use std::convert::From;

#[derive(Debug, Display)]
pub enum LyriekError {
    PlayerError(String),
    #[display(fmt = "Player not found")]
    PlayerNotFound,
    Communication(String),
    #[display(fmt = "Arist(s) not found")]
    ArtistNotFound,
    #[display(fmt = "Song title not found")]
    TitleNotFound,
    /// The API response was succesful, but there were no lyrics found
    LyricsNotFound,
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

impl From<reqwest::Error> for LyriekError {
    fn from(error: reqwest::Error) -> LyriekError {
        error!("API request failed: {}", error);
        LyriekError::LyricsNotFound
    }
}
