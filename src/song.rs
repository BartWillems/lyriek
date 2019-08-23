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

#[derive(Clone)]
pub struct Song {
    pub title: String,
    pub artists: String,
    pub lyrics: Option<String>,
    pub hash: String,
}

impl Song {
    pub fn new() -> Song {
        Song {
            artists: String::from(""),
            title: String::from(""),
            lyrics: None,
            hash: String::from(""),
        }
    }

    pub fn get_playing_song<'a>(&self, player: &mpris::Player<'a>) -> Option<Song> {
        let metadata = player
            .get_metadata()
            .or_else(|e| {
                debug!("unable to fetch the player metadata: {}", e);
                Err(e)
            })
            .ok()?;

        debug!("mpris metadata {:#?}", metadata);

        let mut song = Song {
            artists: metadata
                .artists()
                .ok_or("artist not found")
                .ok()?
                .join(", "),
            title: metadata.title().ok_or("title not found").ok()?.to_owned(),
            lyrics: None,
            hash: metadata.track_id().to_owned(),
        };

        match song.get_lyrics() {
            Err(e) => debug!("unable to fetch lyrics: {}", e),
            _ => {}
        };

        Some(song)
    }

    fn get_lyrics(&mut self) -> Result<(), Box<Error>> {
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
