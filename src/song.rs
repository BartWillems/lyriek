extern crate url;

use std::error::Error;

#[derive(Deserialize)]
struct ApiResponse {
    result: ApiResult,
}

#[derive(Deserialize)]
struct ApiResult {
    track: Track,
}

#[derive(Deserialize)]
struct Track {
    text: String,
}

#[derive(Default, Clone)]
pub struct Song {
    pub title: String,
    pub artists: String,
    pub album: Option<String>,
    pub album_art_url: Option<url::Url>,
    pub url: Option<url::Url>,
    pub lyrics: Option<String>,
}

impl Song {
    pub fn new() -> Self {
        Song::default()
    }

    pub fn new_from_metadata(metadata: &mpris::Metadata) -> Option<Self> {
        debug!("mpris metadata {:#?}", metadata);

        let mut song = Song {
            artists: metadata.artists()?.join(", "),
            title: metadata.title()?.to_owned(),
            lyrics: None,
            album: metadata.album_name().map(|s| s.to_string()),
            album_art_url: metadata.art_url().and_then(|s| url::Url::parse(s).ok()),
            url: metadata.url().and_then(|s| url::Url::parse(s).ok()),
        };

        // Sometimes MPRIS gives an empty response
        if song.artists.is_empty() || song.title.is_empty() {
            return None;
        }

        if let Err(e) = song.get_lyrics() {
            debug!("unable to fetch lyrics: {}", e);
        }

        Some(song)
    }

    pub fn get_playing_song<'a>(player: &mpris::Player<'a>) -> Option<Song> {
        let metadata = player
            .get_metadata()
            .or_else(|e| {
                debug!("unable to fetch the player metadata: {}", e);
                Err(e)
            })
            .ok()?;

        return Song::new_from_metadata(&metadata);
    }

    fn get_lyrics_api_uri(&self) -> Result<url::Url, Box<dyn Error>> {
        use url::Url;
        let mut url = Url::parse("https://orion.apiseeds.com/api/music/lyric")?;

        url.path_segments_mut()
            .map_err(|_| "cannot be base")?
            .push(&self.artists)
            .push(&self.title);
        url.query_pairs_mut().append_pair(
            "apikey",
            "DasGEcpYgIQRlcEEs0reSyuvn9uIcvisOaFW1QiVK7uS3mPpYL7Qb25YmPIVl60r",
        );

        Ok(url)
    }

    fn get_lyrics(&mut self) -> Result<(), Box<dyn Error>> {
        let url = &self.get_lyrics_api_uri()?;

        debug!("fetching lyrics from {}", url.as_str());
        let resp: ApiResponse = reqwest::get(url.as_str())?.json().or_else(|e| {
            debug!("unable to fetch lyrics: {}", e);
            self.lyrics = None;
            Err("lyrics not found")
        })?;

        self.lyrics = Some(resp.result.track.text);
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
            lyrics: None,
            album: None,
            album_art_url: None,
            url: None,
        };

        let uri = song.get_lyrics_api_uri().unwrap();
        // This is to make sure the artist & song title aren't switched
        assert_eq!(uri.path(), "/api/music/lyric/Opeth/Blackwater%20Park");
    }
}
