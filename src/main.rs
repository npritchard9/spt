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
        writeln!(f, "{} | {} | {}", self.name, self.artist, self.album)
    }
}

#[tokio::main]
async fn main() {
    let db = db::get_db().await.expect("The db should exist");
    let db_token = db::select_token(&db).await.expect("A db token to exist");
    match db_token {
        Some(_) => {
            println!("You have a token already.",)
        }
        None => {
            let new_token = gsat().await.unwrap();
            db::insert_token(&db, new_token.clone()).await.unwrap();
            println!("Fetched an access token.");
        }
    }

    if check_refresh(&db)
        .await
        .expect("Should be able to check refresh")
    {
        println!("Refreshing token...");
        if let Some(ref token) = db_token {
            let refreshed_token = refresh_token(token.refresh_token.clone()).await.unwrap();
            update_token(&db, refreshed_token.access_token.clone())
                .await
                .expect("Should be able to update the db with the new token");
        }
    }

    let db_token = db::select_token(&db)
        .await
        .expect("A db token to exist")
        .expect("The new db token to exist");

    let access_token = Some(SpotifyAccessToken {
        access_token: db_token.access_token,
        token_type: db_token.token_type,
        scope: db_token.scope,
        expires_in: db_token.expires_in,
        refresh_token: db_token.refresh_token,
    });

    let matches = command!()
        .arg(arg!(-p --playlist <NAME> "Search a playlist").required(false))
        .arg(arg!(-a --playlists ... "View all playlists").required(false))
        .arg(arg!(-n --next ... "Skip to next song").required(false))
        .arg(arg!(-c --current ... "View current song").required(false))
        .arg(arg!(-l --logout ... "Logout").required(false))
        .get_matches();
    if let Some(name) = matches.get_one::<String>("playlist") {
        println!("Searching for: {}", name.trim());
        if let Some(token) = access_token.clone() {
            println!("token: {}", token.access_token.clone());
            let playlists = get_all_playlists(token.clone())
                .await
                .expect("There should be playlists to return");
            // for playlist in playlists.iter() {
            //     println!("{} | {}", playlist.name, playlist.owner)
            // }

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
        }
    };
    match matches.get_one::<u8>("playlists") {
        Some(0) => (),
        _ => {
            if let Some(token) = access_token.clone() {
                println!("token: {}", token.access_token.clone());
                let playlists = get_all_playlists(token)
                    .await
                    .expect("There should be playlists to return");
                for playlist in playlists {
                    println!("{} | {}", playlist.name, playlist.owner)
                }
            }
        }
    };
    match matches.get_one::<u8>("next") {
        Some(0) => (),
        _ => {
            if let Some(token) = access_token.clone() {
                println!("token: {}", token.access_token.clone());
                let song = skip_to_next(token)
                    .await
                    .expect("There should be a next song");
                println!("{song}")
            }
        }
    };
    match matches.get_one::<u8>("current") {
        Some(0) => (),
        _ => {
            if let Some(token) = access_token.clone() {
                println!("token: {}", token.access_token.clone());
                let song = get_currently_playing(token)
                    .await
                    .expect("There should be a current song");
                println!("{song}")
            }
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
