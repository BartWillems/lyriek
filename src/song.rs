use std::convert::TryFrom;
use std::error::Error;

use url::Url;

use crate::errors::LyriekError;

#[derive(Deserialize)]
struct ApiResponse {
    lyrics: String,
}

#[derive(Clone, Debug)]
pub enum Lyrics {
    Loading,
    Found(String),
    NotFound,
}

impl Default for Lyrics {
    fn default() -> Self {
        Lyrics::Loading
    }
}

#[derive(Default, Clone)]
pub struct Song {
    pub title: String,
    pub artists: String,
    pub album: Option<String>,
    pub album_art_url: Option<url::Url>,
    pub url: Option<url::Url>,
    pub lyrics: Lyrics,
}

impl TryFrom<&mpris::Metadata> for Song {
    type Error = LyriekError;

    fn try_from(metadata: &mpris::Metadata) -> Result<Self, Self::Error> {
        let song = Song {
            artists: metadata
                .artists()
                .ok_or(LyriekError::ArtistNotFound)?
                .join(", "),
            title: metadata
                .title()
                .ok_or(LyriekError::TitleNotFound)?
                .to_owned(),
            lyrics: Lyrics::default(),
            album: metadata.album_name().map(|s| s.to_string()),
            album_art_url: metadata.art_url().and_then(|s| Url::parse(s).ok()),
            url: metadata.url().and_then(|s| Url::parse(s).ok()),
        };

        Ok(song)
    }
}

impl Song {
    pub fn new() -> Self {
        Song::default()
    }

    /// Returns the current playing song according to the mpris player
    pub fn get_playing_song<'a>(player: &mpris::Player<'a>) -> Result<Song, LyriekError> {
        let metadata = player.get_metadata().or_else(|e| {
            debug!("unable to fetch the player metadata: {}", e);
            Err(e)
        })?;

        Song::try_from(&metadata)
    }

    /// returns the url::Url to fetch the lyrics for the current song
    /// returns an error if the url can't be parsed
    fn get_lyrics_api_uri(&self) -> url::Url {
        let mut url = Url::parse("https://api.lyrics.ovh/v1").expect("invalid api base url");

        url.path_segments_mut()
            .expect("url can not be base")
            .push(&self.artists)
            .push(&self.title);

        url
    }

    /// sets the lyrics for the song
    pub fn get_lyrics(&mut self) -> Result<(), Box<dyn Error>> {
        let url = &self.get_lyrics_api_uri();

        debug!("fetching lyrics from {}", url.as_str());

        let resp: ApiResponse = reqwest::blocking::get(url.as_str())
            .map_err(|e| {
                error!("lyrics api error: {}", e);
                self.lyrics = Lyrics::NotFound;
                e
            })?
            .json()
            .or_else(|e| {
                debug!("unable to fetch lyrics: {}", e);
                self.lyrics = Lyrics::NotFound;
                Err("lyrics not found")
            })?;

        // For some reason, this lyrics api adds too much new lines
        self.lyrics = Lyrics::Found(resp.lyrics.replace("\n\n", "\n"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lyrics_api_path_ordering() {
        let song: Song = Song {
            title: String::from("Blackwater Park"),
            artists: String::from("Opeth"),
            lyrics: Lyrics::default(),
            album: None,
            album_art_url: None,
            url: None,
        };

        let uri = song.get_lyrics_api_uri();
        // This is to make sure the artist & song title aren't switched
        assert_eq!(uri.path(), "/v1/Opeth/Blackwater%20Park");
    }

    #[test]
    fn test_lyrics_api_url_encoding() {
        let mut song: Song = Song::new();
        song.artists = String::from("Slayer");
        song.title = String::from("Metal Storm / Face the Slayer");

        let uri = song.get_lyrics_api_uri();

        assert_eq!(
            uri.path(),
            "/v1/Slayer/Metal%20Storm%20%2F%20Face%20the%20Slayer"
        );
    }
}
