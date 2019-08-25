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
use relm::{Channel, Relm, Widget};
use relm_derive::widget;
use std::thread;
use v_htmlescape::escape;

use assets::Assets;

mod assets;
mod player;
mod song;

pub struct Model {
    _channel: Channel<Msg>,
    lyrics: String,
    title: String,
    artists: String,
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
            if let Err(e) = player::get_events(&sender) {
                let _ = sender.send(Msg::Error(format!("{}", e)));
            }

            thread::sleep(std::time::Duration::from_secs(1));
        });

        Model {
            _channel: channel,
            lyrics: "".to_string(),
            title: "".to_string(),
            artists: "".to_string(),
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
                self.model.title = format!(
                    "<span size=\"xx-large\" weight=\"bold\">{}</span>",
                    escape(&song.title)
                );
                self.model.artists =
                    format!("<span size=\"x-large\">{}</span>", escape(&song.artists));
            }
            Msg::Error(e) => {
                self.model.lyrics = e;
                self.model.title = String::from("");
                self.model.artists = String::from("");
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
                        gtk::Box {
                            orientation: Vertical,
                            gtk::Label {
                                markup: &self.model.title,
                            },
                            gtk::Label {
                                markup: &self.model.artists,
                            },
                        },
                        gtk::Box {
                            orientation: Vertical,
                            vexpand: true,
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
            },
        }
    }
}

fn main() {
    env_logger::init();

    Window::run(()).expect("Lyriek startup failed");
}
