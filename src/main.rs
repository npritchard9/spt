use clap::{arg, command};
use std::fmt::Display;

mod auth;
mod db;
mod spotify;

use auth::*;
use db::{check_refresh, handle_refresh_token};
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
    let mut access_token: Option<SpotifyAccessToken> = None;
    let db_token = db::select_token(&db).await.unwrap();
    match db_token {
        Some(token) => {
            access_token = Some(SpotifyAccessToken {
                access_token: token.access_token,
                token_type: token.token_type,
                scope: token.scope,
                expires_in: token.expires_in,
                refresh_token: token.refresh_token,
            });
            println!("You have a token already.",)
        }
        None => {
            let new_token = gsat().await.unwrap();
            db::insert_token(&db, new_token.clone()).await.unwrap();
            access_token = Some(new_token.clone());
            println!("Fetched an access token.");
        }
    }

    if check_refresh(&db)
        .await
        .expect("Should be able to check refresh")
    {
        if let Some(ref mut token) = access_token.clone() {
            let refreshed_response_token =
                refresh_token(token.refresh_token.clone()).await.unwrap();
            handle_refresh_token(&db, refreshed_response_token.access_token.clone())
                .await
                .expect("Should be able to update the db with the new token");
            token.access_token = refreshed_response_token.access_token;
        }
    }
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
