use clap::{arg, command};
use std::{
    fmt::Display,
    io::{stdin, stdout, Write},
};

mod auth;
mod db;
mod spotify;

use auth::*;
use db::{check_refresh, update_token, ClientCredentials};
use spotify::*;

#[derive(Debug)]
pub struct Playlist {
    name: String,
    owner: String,
    id: String,
}

#[derive(Debug)]
pub struct Song {
    name: String,
    album: String,
    artist: String,
    uri: String,
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {} | {}", self.name, self.artist, self.album)
    }
}

#[tokio::main]
async fn main() {
    let db = db::get_db().await.expect("The db should exist");
    let creds = db::select_credentials(&db)
        .await
        .expect("Credentials to exist");
    if creds.is_none() {
        let mut input = String::new();
        println!("Enter spotify client id:");
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        let client_id = input.trim();
        let mut input = String::new();
        println!("Enter spotify client secret:");
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        let secret = input.trim();
        db::insert_client_credentials(
            &db,
            ClientCredentials {
                client_id: client_id.to_string(),
                secret: secret.to_string(),
            },
        )
        .await
        .expect("Should be able to insert client creds");
    }
    let creds = db::select_credentials(&db)
        .await
        .expect("Credentials to exist")
        .expect("And to be real");
    let db_token = db::select_token(&db).await.expect("A db token to exist");
    if db_token.is_none() {
        let new_token = gsat(creds.client_id.clone(), creds.secret.clone())
            .await
            .unwrap();
        db::insert_token(&db, new_token).await.unwrap();
        println!("Fetched a new access token.");
    }
    if check_refresh(&db)
        .await
        .expect("Should be able to check refresh")
    {
        println!("Refreshing token...");
        if let Some(token) = db_token {
            let refreshed_token = refresh_token(
                token.refresh_token,
                creds.client_id.clone(),
                creds.secret.clone(),
            )
            .await
            .unwrap();
            update_token(&db, refreshed_token.access_token.clone())
                .await
                .expect("Should be able to update the db with the new token");
        }
    }

    let token = db::select_token(&db)
        .await
        .expect("A db token to exist")
        .expect("The new db token to exist by now");

    let matches = command!()
        .arg(arg!(-l --playlist <NAME> "Search a playlist").required(false))
        .arg(arg!(-a --playlists ... "View all playlists").required(false))
        .arg(arg!(-x --pause ... "Pause playback").required(false))
        .arg(arg!(-r --resume ... "Resume playback").required(false))
        .arg(arg!(-e --start <QUERY> "Start playing QUERY related songs").required(false))
        .arg(arg!(-n --next ... "Skip to next song").required(false))
        .arg(arg!(-p --prev ... "Skip to previous song").required(false))
        .arg(arg!(-c --current ... "View current song").required(false))
        .arg(arg!(-m --shuffle <STATE> "Shuffle Y/N").required(false))
        .arg(arg!(-s --search <QUERY> "Search spotify").required(false))
        .arg(arg!(-u --update <QUERY> "Add tracks to a playlist").required(false))
        .arg(arg!(-q --logout ... "Logout").required(false))
        .get_matches();
    if let Some(name) = matches.get_one::<String>("playlist") {
        let playlists = get_all_playlists(token.clone())
            .await
            .expect("There should be playlists to return");

        let name = name.trim();
        let curr_playlist = playlists
            .iter()
            .find(|p| p.name == name)
            .expect("User must enter a valid playlist name");

        let songs = get_playlist(token.clone(), curr_playlist.id.clone())
            .await
            .expect("There should be playlists to return");
        for song in songs {
            println!("{song}")
        }
    };
    if let Some(query) = matches.get_one::<String>("search") {
        let query = query.trim();
        let search_res = search_for_item(token.clone(), query)
            .await
            .expect("There should be search results to return");

        for (i, song) in search_res.iter().enumerate() {
            println!("{}. {song}", i + 1);
        }
        println!("\nEnter a number to add a song to queue, or q to exit");
        let mut input = String::new();
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        match input {
            "q" => (),
            _ => {
                let index = input.parse::<usize>().unwrap();
                let uri = search_res[index - 1].uri.clone();
                add_to_queue(token.clone(), uri.clone())
                    .await
                    .expect("Should be able to add to queue");
            }
        }
    };
    if let Some(query) = matches.get_one::<String>("update") {
        let query = query.trim();
        let search_res = search_for_item(token.clone(), query)
            .await
            .expect("There should be search results to return");

        println!("Songs:");
        for (i, song) in search_res.iter().enumerate() {
            println!("{}. {song}", i + 1);
        }
        println!("Your playlists:");
        let playlists = get_all_playlists(token.clone())
            .await
            .expect("There should be playlists to return");
        for playlist in playlists.iter() {
            println!("{} | {}", playlist.name, playlist.owner)
        }
        println!("\nEnter the name of a playlist to add songs to, or q to exit");
        let mut input = String::new();
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        let playlist_name = input.trim();
        //not sure if this return works
        if playlist_name == "q" {
            ()
        }
        let curr_playlist = playlists.iter().find(|p| p.name == playlist_name).unwrap();
        let pid = curr_playlist.id.clone();
        let prompt = format!(
            "\nEnter comma-separated numbers to add songs to {}, or q to exit",
            playlist_name
        );
        println!("{}", prompt);
        let mut input = String::new();
        stdout().flush().unwrap();
        stdin().read_line(&mut input).unwrap();
        let ids = input.trim();
        match ids {
            "q" => (),
            _ => {
                let uris: Vec<String> = ids
                    .split(",")
                    .map(|id| id.parse::<usize>().unwrap() - 1)
                    .map(|id| search_res[id].uri.clone())
                    .collect();
                add_to_playlist(token.clone(), pid, uris)
                    .await
                    .expect("To be able to add songs to playlist");
            }
        }
    };
    if let Some(query) = matches.get_one::<String>("start") {
        let query = query.trim();
        let search_res = search_for_item(token.clone(), query)
            .await
            .expect("There should be playlists to return");
        println!("Playing the following songs:");
        for song in search_res.iter() {
            println!("{song}");
        }
        let mut uris = vec![];
        for song in search_res {
            uris.push(song.uri);
        }
        start_playing(token.clone(), uris)
            .await
            .expect("Should be able to play the song");
    };
    if let Some(state) = matches.get_one::<String>("shuffle") {
        let state = state.trim().to_lowercase();
        let shuffle_state = {
            if state == "y" {
                true
            } else {
                false
            }
        };
        shuffle(token.clone(), shuffle_state)
            .await
            .expect("Should be able to shuffle");
    };
    match matches.get_one::<u8>("playlists") {
        Some(0) => (),
        _ => {
            let playlists = get_all_playlists(token.clone())
                .await
                .expect("There should be playlists to return");
            for playlist in playlists {
                println!("{} | {}", playlist.name, playlist.owner)
            }
        }
    };
    match matches.get_one::<u8>("pause") {
        Some(0) => (),
        _ => {
            pause(token.clone()).await.expect("Should be able to pause");
        }
    };
    match matches.get_one::<u8>("resume") {
        Some(0) => (),
        _ => {
            resume(token.clone())
                .await
                .expect("Should be able to resume");
        }
    };
    match matches.get_one::<u8>("next") {
        Some(0) => (),
        _ => {
            skip_to_next(token.clone())
                .await
                .expect("There should be a next song");
        }
    };
    match matches.get_one::<u8>("prev") {
        Some(0) => (),
        _ => {
            skip_to_prev(token.clone())
                .await
                .expect("There should be a previous song");
        }
    };
    match matches.get_one::<u8>("current") {
        Some(0) => (),
        _ => {
            let song = get_currently_playing(token.clone())
                .await
                .expect("There should be a current song");
            println!("{song}")
        }
    };
    match matches.get_one::<u8>("logout") {
        Some(0) => (),
        _ => {
            db::delete_credentials(&db)
                .await
                .expect("Was able to delete credentials");
            db::delete_token(&db)
                .await
                .expect("User was able to logout");
            println!("Logged out successfully.")
        }
    };
}
