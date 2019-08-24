extern crate env_logger;
extern crate gdk_pixbuf;
extern crate gtk;
extern crate mpris;
extern crate serde;
extern crate v_htmlescape;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate rust_embed;

use gtk::Orientation::Vertical;
use gtk::{GtkWindowExt, Inhibit, LabelExt, OrientableExt, ScrollableExt, SpinnerExt, WidgetExt};
use mpris::PlayerFinder;
use relm::{Channel, Relm, Widget};
use relm_derive::widget;
use std::error::Error;
use std::thread;
use v_htmlescape::escape;

use assets::Assets;

mod assets;
mod song;

pub struct Model {
    _channel: Channel<Msg>,
    lyrics: String,
    header: String,
    is_loading: bool,
    logo: Option<gdk_pixbuf::Pixbuf>,
}

#[derive(Clone, Msg)]
pub enum Msg {
    Quit,
    Song(song::Song),
    Error(String),
    StopLoading,
    StartLoading,
}

#[widget]
impl Widget for Window {
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        let stream = relm.stream().clone();
        // Create a channel to be able to send a message from another thread.
        let (channel, sender) = Channel::new(move |msg| {
            stream.emit(msg);
        });

        thread::spawn(move || loop {
            let start_loop = || -> Result<(), Box<dyn Error>> {
                let player = get_mpris_player().or_else(|e| {
                    trace!("attempting to fetch the player");
                    sender.send(Msg::StopLoading)?;
                    sender.send(Msg::Error(String::from("no player found")))?;
                    Err(e)
                })?;

                match song::Song::new().get_playing_song(&player) {
                    Some(song) => sender.send(Msg::Song(song))?,
                    None => sender.send(Msg::Error(String::from("song not found")))?,
                }
                sender.send(Msg::StopLoading)?;

                let events = player
                    .events()
                    .or_else(|e| Err(format!("unable to start event stream: {}", e)))?;

                for event in events {
                    match event {
                        Ok(event) => {
                            debug!("mpris event: {:#?}", event);

                            match event {
                                mpris::Event::TrackChanged(_) => {
                                    sender.send(Msg::StartLoading)?;
                                    match song::Song::new().get_playing_song(&player) {
                                        Some(song) => {
                                            sender.send(Msg::Song(song))?;
                                        }
                                        None => {
                                            sender
                                                .send(Msg::Error(String::from("song not found")))?;
                                        }
                                    }
                                    sender.send(Msg::StopLoading)?;
                                }
                                mpris::Event::PlayerShutDown => sender
                                    .send(Msg::Error("connection to player lost".to_owned()))?,
                                _ => {}
                            }
                        }
                        Err(err) => {
                            sender.send(Msg::Error(format!("D-Bus error: {}", err)))?;
                            break;
                        }
                    }
                }

                debug!("connection to player lost... attempting to reconnect...");
                Ok(())
            };

            let _ = match start_loop() {
                Err(e) => sender.send(Msg::Error(format!("{}", e))),
                Ok(()) => Ok(()),
            };

            thread::sleep(std::time::Duration::from_secs(1));
        });

        Model {
            _channel: channel,
            lyrics: "".to_string(),
            header: "".to_string(),
            is_loading: true,
            logo: Assets::get_logo_pixbuf(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::Song(song) => {
                match song.lyrics {
                    Some(lyrics) => {
                        self.model.lyrics =
                            format!("<span size=\"large\">{}</span>", escape(&lyrics))
                    }
                    None => self.model.lyrics = String::from("lyrics not found :("),
                };
                self.model.header = format!(
                    "<span size=\"xx-large\" weight=\"bold\">{} - {}</span>",
                    escape(&song.artists),
                    escape(&song.title)
                );
            }
            Msg::Error(e) => {
                self.model.lyrics = e;
                self.model.header = String::from("");
            }
            Msg::StopLoading => self.model.is_loading = false,
            Msg::StartLoading => self.model.is_loading = true,
        }
    }

    view! {
        gtk::Window {
            title: "lyriek",
            icon: self.model.logo.as_ref(),
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
            property_width_request: 760,
            property_height_request: 600,
            gtk::ScrolledWindow {
                gtk::Viewport {
                    hscroll_policy: gtk::ScrollablePolicy::Natural,
                    vscroll_policy: gtk::ScrollablePolicy::Natural,
                    gtk::Box {
                        property_margin: 15,
                        orientation: Vertical,
                        gtk::Label {
                            markup: &self.model.header,
                        },
                        gtk::Spinner {
                            property_active: self.model.is_loading,
                        },
                        gtk::Label {
                            selectable: true,
                            markup: &self.model.lyrics,
                        },
                    },
                },
            },
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

fn main() {
    env_logger::init();

    Window::run(()).expect("Lyriek startup failed");
}
