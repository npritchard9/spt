use clap::{arg, command};
use std::fmt::Display;

mod auth;
mod db;
mod spotify;

use auth::*;
use db::{check_refresh, update_token};
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
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {} | {}", self.name, self.artist, self.album)
    }
}

#[tokio::main]
async fn main() {
    let db = db::get_db().await.expect("The db should exist");
    let db_token = db::select_token(&db).await.expect("A db token to exist");
    if db_token.is_none() {
        let new_token = gsat().await.unwrap();
        db::insert_token(&db, new_token).await.unwrap();
        println!("Fetched a new access token.");
    }
    if check_refresh(&db)
        .await
        .expect("Should be able to check refresh")
    {
        println!("Refreshing token...");
        if let Some(token) = db_token {
            let refreshed_token = refresh_token(token.refresh_token).await.unwrap();
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
        .arg(arg!(-n --next ... "Skip to next song").required(false))
        .arg(arg!(-p --prev ... "Skip to previous song").required(false))
        .arg(arg!(-c --current ... "View current song").required(false))
        .arg(arg!(-s --search <QUERY> "Search spotify").required(false))
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
            .expect("There should be playlists to return");

        for song in search_res {
            println!("{song}")
        }
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
            db::delete_token(&db)
                .await
                .expect("User was able to logout");
            println!("Logged out successfully.")
        }
    };
}
