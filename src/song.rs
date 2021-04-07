use std::convert::TryFrom;
use std::fmt::Debug;

use url::Url;

use crate::errors::LyriekError;

#[derive(Debug, Deserialize)]
struct ApiResponse<T: Debug> {
    result: T,
}

#[derive(Debug, Deserialize)]
struct ApiSearchResponse {
    artist: String,
    id_artist: usize,
    track: String,
    id_track: usize,
    album: String,
    id_album: usize,
    api_lyrics: String,
}

#[derive(Debug, Deserialize)]
struct ApiLyricsResponse {
    artist: String,
    id_artist: usize,
    track: String,
    id_track: usize,
    album: String,
    id_album: usize,
    lyrics: String,
    /// URL to this resource
    api_lyrics: String,
}

// #[derive(Deserialize)]
// struct ApiResponse {
//     result: ApiResult,
//     // lyrics: String,
// }

// #[derive(Deserialize)]
// struct ApiResult {
//     track: Track,
// }

// #[derive(Deserialize)]
// struct Track {
//     text: String,
// }

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

    /// Try to load the lyrics from the API
    fn search_lyrics(&mut self) -> Result<String, LyriekError> {
        let mut url = Url::parse("https://api.happi.dev/v1/music").expect("invalid base API url");

        url.query_pairs_mut()
            .append_pair("q", &format!("{} {}", self.artists, self.title))
            .append_pair("limit", "1")
            .append_pair("lyrics", "1")
            .append_pair("type", "track")
            // TODO: this is a free tier API key, I should replace this with a server of my own so I don't have to push this to github
            // For anyone who reads this: Don't steal this API key. (Or do, I don't really care atm)
            .append_pair(
                "apikey",
                "84cbb8oiGQIoD68KLQSscMr3jNYWwvQSFFhpq46tNfHcRQComN1lMd4d",
            );

        let mut resp: ApiResponse<Vec<ApiSearchResponse>> = reqwest::blocking::get(url)?.json()?;

        let lyrics_url = resp
            .result
            .pop()
            .ok_or_else(|| LyriekError::LyricsNotFound)?;

        let lyrics: ApiResponse<ApiLyricsResponse> = reqwest::blocking::get(
            lyrics_url.api_lyrics
                + "?apikey=84cbb8oiGQIoD68KLQSscMr3jNYWwvQSFFhpq46tNfHcRQComN1lMd4d",
        )?
        .json()?;

        Ok(lyrics.result.lyrics)
    }

    pub fn get_lyrics(&mut self) -> Result<(), LyriekError> {
        match self.search_lyrics() {
            Ok(lyrics) => {
                self.lyrics = Lyrics::Found(lyrics);
                Ok(())
            }
            Err(e) => {
                self.lyrics = Lyrics::NotFound;
                return Err(e);
            }
        }
    }
}
