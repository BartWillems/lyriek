extern crate gtk;
extern crate mpris;
extern crate reqwest;
extern crate serde;
extern crate url;
extern crate v_htmlescape;

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate serde_derive;

use mpris::PlayerFinder;

use std::error::Error;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use std::thread;

use gtk::Orientation::Vertical;
use gtk::{GtkWindowExt, Inhibit, LabelExt, OrientableExt, SpinnerExt, WidgetExt};
use relm::{Channel, Relm, Widget};
use relm_derive::widget;

use v_htmlescape::escape;

pub struct Model {
    _channel: Channel<Msg>,
    text: String,
    header: String,
    is_loading: bool,
}

#[derive(Clone, Msg)]
pub enum Msg {
    Quit,
    Song(SongInfo),
    Error(String),
    StopLoading,
    StartLoading,
}

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
pub struct SongInfo {
    title: String,
    artists: String,
    lyrics: String,
    hash: String,
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        let stream = relm.stream().clone();
        // Create a channel to be able to send a message from another thread.
        let (channel, sender) = Channel::new(move |msg| {
            stream.emit(msg);
        });

        thread::spawn(move || {
            let player;
            let mpris_res = get_mpris_player();
            match mpris_res {
                Ok(p) => player = p,
                Err(_) => {
                    sender.send(Msg::StopLoading).expect("send message");
                    sender
                        .send(Msg::Error(String::from("no player found")))
                        .expect("send message");
                    return;
                }
            }

            match get_current_song(&player) {
                Ok(song) => {
                    sender.send(Msg::Song(song)).expect("send message");
                }
                Err(e) => {
                    sender
                        .send(Msg::Error(format!("{}", e)))
                        .expect("send message");
                }
            }
            sender.send(Msg::StopLoading).expect("send message");

            let events = player.events().expect("Could not start event stream");

            for event in events {
                match event {
                    Ok(event) => {
                        debug!("event: {:#?}", event);

                        match event {
                            mpris::Event::TrackChanged(_) => {
                                sender.send(Msg::StartLoading).expect("send message");
                                match get_current_song(&player) {
                                    Ok(song) => {
                                        sender.send(Msg::Song(song)).expect("send message");
                                    }
                                    Err(e) => {
                                        sender
                                            .send(Msg::Error(format!("{}", e)))
                                            .expect("send message");
                                    }
                                }
                                sender.send(Msg::StopLoading).expect("send message");
                            }
                            mpris::Event::PlayerShutDown => sender
                                .send(Msg::Error(
                                    "player shutdown, lyriek restart is required".to_owned(),
                                ))
                                .expect("send message"),
                            _ => {}
                        }
                    }
                    Err(err) => {
                        sender
                            .send(Msg::Error(format!("D-Bus error: {}", err)))
                            .expect("send message");
                        break;
                    }
                }
            }
        });
        Model {
            _channel: channel,
            text: "".to_string(),
            header: "".to_string(),
            is_loading: true,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::Song(song) => {
                self.model.text = format!("<span size=\"large\">{}</span>", escape(&song.lyrics));
                self.model.header = format!(
                    "<span size=\"xx-large\" weight=\"bold\">{} - {}</span>",
                    escape(&song.artists),
                    escape(&song.title)
                );
            }
            Msg::Error(e) => self.model.text = e,
            Msg::StopLoading => self.model.is_loading = false,
            Msg::StartLoading => self.model.is_loading = true,
        }
    }

    view! {
        gtk::Window {
            title: "lyriek",
            // icon_from_file: &std::path::Path::new("./screenshots/initial-release.png"),
            gtk::Box {
                // spacing: 30,
                orientation: Vertical,
                gtk::Label {
                    markup: &self.model.header,
                },
                gtk::Spinner {
                    property_active: self.model.is_loading,
                },
                gtk::Label {
                    selectable: true,
                    markup: &self.model.text,
                },
            },
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}

fn get_mpris_player<'a>() -> Result<mpris::Player<'a>, Box<dyn Error>> {
    let player = PlayerFinder::new()
        .or_else(|e| Err(format!("Unable to connect to the dbus player: {}", e)))?
        .find_active()
        .or_else(|_| Err("no active player found"))?;

    Ok(player)
}

fn get_current_song<'a>(player: &mpris::Player<'a>) -> Result<SongInfo, Box<dyn Error>> {
    let metadata = player.get_metadata().expect("Unable to fetch metadata");

    let title = metadata.title().ok_or("title not found")?;
    let artists = metadata.artists().ok_or("artist not found")?.join(", ");

    let lyrics = fetch_lyrics(&artists, title)?;

    Ok(SongInfo {
        title: title.to_owned(),
        artists: artists,
        lyrics: lyrics,
        hash: metadata.track_id().to_owned(),
    })
}

fn fetch_lyrics(artists: &str, title: &str) -> Result<String, Box<dyn Error>> {
    use url::Url;
    let mut url = Url::parse("https://orion.apiseeds.com/api/music/lyric")?;

    url.path_segments_mut()
        .map_err(|_| "cannot be base")?
        .push(artists)
        .push(title);
    url.query_pairs_mut().append_pair(
        "apikey",
        "DasGEcpYgIQRlcEEs0reSyuvn9uIcvisOaFW1QiVK7uS3mPpYL7Qb25YmPIVl60r",
    );

    debug!("fetching lyrics from {}", url.as_str());
    let resp: ApiResponse = reqwest::get(url.as_str())?
        .json()
        .or_else(|_| Err("lyrics not found"))?;
    Ok(resp.result.track.text)
}

fn main() {
    env_logger::init();

    Win::run(()).expect("Win::run failed");
}
