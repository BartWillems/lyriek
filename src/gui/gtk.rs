use gtk::Orientation::Vertical;
use gtk::{GtkWindowExt, Inhibit, LabelExt, OrientableExt, ScrollableExt, SpinnerExt, WidgetExt};
use relm::{Channel, Relm, Widget};
use relm_derive::widget;
use std::thread;
use v_htmlescape::escape;

use crate::assets;
use crate::player;
use crate::song::Lyrics;
use crate::Msg;

pub struct Model {
    _channel: Channel<Msg>,
    lyrics: String,
    title: String,
    artists: String,
    is_loading: bool,
    logo: Option<gdk_pixbuf::Pixbuf>,
}

// #[derive(Clone, Msg)]
// pub enum Msg {
//     Quit,
//     Song(Song),
//     Error(String),
//     StopLoading,
//     StartLoading,
// }

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
                let res = sender.send(Msg::Error(format!("{}", e)));
                if res.is_err() {
                    error!("unable to send an error message to the client: {}", e);
                }
            }

            thread::sleep(std::time::Duration::from_secs(1));
        });

        Model {
            _channel: channel,
            lyrics: String::new(),
            title: String::new(),
            artists: String::new(),
            is_loading: true,
            logo: assets::get_logo_pixbuf(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::Song(song) => {
                match song.lyrics {
                    Lyrics::Loading => self.model.lyrics = String::from("Loading..."),
                    Lyrics::NotFound => self.model.lyrics = String::from("lyrics not found :("),
                    Lyrics::Found(lyrics) => {
                        self.model.lyrics =
                            format!("<span size=\"large\">{}</span>", escape(&lyrics))
                    }
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

pub fn launch() {
    Window::run(()).expect("Lyriek startup failed");
}
