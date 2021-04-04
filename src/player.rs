use std::convert::TryFrom;

use mpris::PlayerFinder;

use crate::errors::LyriekError;
use crate::song;
use crate::Msg;

fn get_mpris_player<'a>() -> Result<mpris::Player<'a>, LyriekError> {
    let players = PlayerFinder::new()?.find_all()?;

    debug!("List of players: {:?}", players);

    for player in players {
        info!("Found player: {}", player.bus_name());
        // some players can't send event streams
        if player.events().is_ok() {
            return Ok(player);
        }
        error!("player {} can't send events", player.bus_name());
    }

    Err(LyriekError::PlayerNotFound)
}

pub fn get_events(sender: &relm::Sender<Msg>) -> Result<(), LyriekError> {
    let player = get_mpris_player().or_else(|e| {
        trace!("attempting to fetch the player");
        sender.send(Msg::StopLoading)?;
        sender.send(Msg::Error(e.to_string()))?;
        Err(e)
    })?;

    trace!("acquired mpris client");

    // The playing song has to be acquired manually as this doesn't receive an event
    match song::Song::get_playing_song(&player) {
        Ok(mut song) => {
            sender.send(Msg::Song(song.clone()))?;
            if let Ok(()) = song.get_lyrics() {
                sender.send(Msg::Song(song))?;
            } else {
                error!("Got error while fetching lyrics...");
            }
        }
        Err(e) => sender.send(Msg::Error(format!("{}", e)))?,
    }

    sender.send(Msg::StopLoading)?;

    let events = player.events()?;

    trace!("listening to mpris event stream");

    for event in events {
        trace!("received mpris event");

        let event = event?;

        debug!("mpris event: {:#?}", event);

        match event {
            mpris::Event::TrackChanged(metadata) => {
                sender.send(Msg::StartLoading)?;
                match song::Song::try_from(&metadata) {
                    Ok(mut song) => {
                        trace!("found a song");
                        sender.send(Msg::Song(song.clone()))?;

                        if let Ok(()) = song.get_lyrics() {
                            sender.send(Msg::Song(song))?;
                        } else {
                            error!("Got error while fetching lyrics...");
                        }
                    }
                    Err(e) => {
                        debug!("No song found, metadata: {:#?}", metadata);
                        sender.send(Msg::Error(format!("{}", e)))?;
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
