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
                                sender.send(Msg::Error(String::from("song not found")))?;
                            }
                        }
                        sender.send(Msg::StopLoading)?;
                    }
                    mpris::Event::PlayerShutDown => {
                        sender.send(Msg::Error("connection to player lost".to_owned()))?
                    }
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
}
