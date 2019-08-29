use mpris::PlayerFinder;
use std::error::Error;

use crate::song;
use crate::Msg;

fn get_mpris_player<'a>() -> Result<mpris::Player<'a>, Box<dyn Error>> {
    let player = PlayerFinder::new()
        .or_else(|e| Err(format!("Unable to connect to the dbus player: {}", e)))?
        .find_active()
        .or_else(|_| Err("no active player found"))?;

    Ok(player)
}

pub fn get_events(sender: &relm::Sender<Msg>) -> Result<(), Box<dyn Error>> {
    let player = get_mpris_player().or_else(|e| {
        trace!("attempting to fetch the player");
        sender.send(Msg::StopLoading)?;
        sender.send(Msg::Error(String::from("no player found")))?;
        Err(e)
    })?;

    trace!("acquired mpris client");

    // The playing song has to be acquired manually as this doesn't receive an event
    match song::Song::get_playing_song(&player) {
        Some(song) => sender.send(Msg::Song(Box::new(song)))?,
        None => sender.send(Msg::Error(String::from("song not found")))?,
    }
    sender.send(Msg::StopLoading)?;

    let events = player
        .events()
        .or_else(|e| Err(format!("unable to start event stream: {}", e)))?;

    trace!("listening to mpris event stream");

    for event in events {
        trace!("received mpris event");

        let event = event.or_else(|err| {
            error!("D-Bus error {}", err);
            return Err(format!("D-Bus error {}", err));
        })?;

        debug!("mpris event: {:#?}", event);

        match event {
            mpris::Event::TrackChanged(metadata) => {
                sender.send(Msg::StartLoading)?;
                match song::Song::new_from_metadata(&metadata) {
                    Some(song) => {
                        trace!("found a song");
                        sender.send(Msg::Song(Box::new(song)))?;
                    }
                    None => {
                        debug!("No song found, metadata: {:#?}", metadata);
                        sender.send(Msg::Error(String::from("song not found")))?;
                    }
                }
                sender.send(Msg::StopLoading)?;
            }
            // When this event arrives, the events iterator also stops and this function returns
            mpris::Event::PlayerShutDown => {
                debug!("connection to player lost...");
                sender.send(Msg::Error("connection to player lost".to_owned()))?
            }
            _ => {}
        }
    }

    trace!("no longer receiving mpris events");

    Ok(())
}
