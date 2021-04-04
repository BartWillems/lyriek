#[cfg(all(feature = "gui-gtk", feature = "gui-iced"))]
compile_error!("feature \"gui-gtk\" and feature \"gui-iced\" cannot be enabled at the same time");

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[cfg(feature = "gui-gtk")]
#[macro_use]
extern crate relm_derive;

#[macro_use]
extern crate rust_embed;

mod assets;
mod errors;
mod gui;
mod player;
mod song;

#[derive(Clone, Msg)]
pub enum Msg {
    Quit,
    Song(song::Song),
    Error(String),
    StopLoading,
    StartLoading,
}

fn main() {
    env_logger::init();

    gui::launch();
}
