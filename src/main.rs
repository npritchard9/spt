use anyhow::anyhow;
use std::{
    fmt::Display,
    io::{stdin, stdout, Write},
    str::FromStr,
};

mod auth;
mod db;
mod spotify;

use auth::*;
use db::{check_refresh, handle_refresh_token};
use spotify::*;

enum Command {
    View = 1,
    Search,
    GetCurrent,
    SkipToNext,
    Logout,
    Quit,
}

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

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Command::View),
            "2" => Ok(Command::Search),
            "3" => Ok(Command::GetCurrent),
            "4" => Ok(Command::SkipToNext),
            "5" => Ok(Command::Logout),
            "6" => Ok(Command::Quit),
            _ => Err(anyhow!("Invalid command.")),
        }
    }
}

fn get_command() -> Command {
    let mut input = String::new();
    println!("What do you want to do?");
    println!("1. View your playlists.");
    println!("2. View songs in a playlist.");
    println!("3. View current song.");
    println!("4. Skip to next song.");
    println!("5. Logout.");
    println!("6. Quit.");
    stdout().flush().unwrap();
    stdin().read_line(&mut input).unwrap();
    let command = input.trim().parse::<Command>().unwrap();
    command
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
    loop {
        match get_command() {
            Command::Search => {
                if let Some(token) = access_token.clone() {
                    println!("token: {}", token.access_token.clone());
                    let playlists = get_all_playlists(token.clone())
                        .await
                        .expect("There should be playlists to return");
                    for playlist in playlists.iter() {
                        println!("{} | {}", playlist.name, playlist.owner)
                    }

                    let mut p_input = String::new();
                    println!("Enter the name of a playlist:");
                    stdout().flush().unwrap();
                    stdin().read_line(&mut p_input).unwrap();
                    let p_name = p_input.trim();
                    let curr_playlist = playlists
                        .iter()
                        .find(|p| p.name == p_name)
                        .expect("User must enter a valid playlist name");

                    let songs = get_playlist(token.clone(), curr_playlist.id.clone())
                        .await
                        .expect("There should be playlists to return");
                    for song in songs {
                        println!("{song}")
                    }
                }
            }
            Command::View => {
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
            Command::GetCurrent => {
                if let Some(token) = access_token.clone() {
                    println!("token: {}", token.access_token.clone());
                    let song = get_currently_playing(token)
                        .await
                        .expect("There should be a current song");
                    println!("{song}")
                }
            }
            Command::SkipToNext => {
                if let Some(token) = access_token.clone() {
                    println!("token: {}", token.access_token.clone());
                    let song = skip_to_next(token)
                        .await
                        .expect("There should be a next song");
                    println!("{song}")
                }
            }
            Command::Logout => {
                db::delete_token(&db)
                    .await
                    .expect("User was able to logout");
                println!("Logged out successfully.")
            }
            Command::Quit => break,
        };
    }
}
