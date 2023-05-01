use anyhow::anyhow;
use db::check_refresh;
use playlist::models::{all_playlists::SpotifyAllPlaylistsRes, playlist::SpotifyPlaylistRes};
use std::{
    io::{stdin, stdout, Write},
    str::FromStr,
};
use surrealdb::{engine::local::Db, Surreal};

mod auth;
mod db;

use crate::auth::*;

enum Command {
    View = 1,
    Search,
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

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Command::View),
            "2" => Ok(Command::Search),
            "3" => Ok(Command::Logout),
            "4" => Ok(Command::Quit),
            _ => Err(anyhow!("Invalid command.")),
        }
    }
}

fn get_command() -> Command {
    let mut input = String::new();
    println!("What do you want to do?");
    println!("1. View your playlists.");
    println!("2. View songs in a playlist.");
    println!("3. Logout.");
    println!("4. Quit.");
    stdout().flush().unwrap();
    stdin().read_line(&mut input).unwrap();
    let command = input.trim().parse::<Command>().unwrap();
    command
}

// this also checks if we need a refresh
async fn get_all_playlists(
    db: &Surreal<Db>,
    spotify_token: SpotifyAccessToken,
) -> Result<Vec<Playlist>, anyhow::Error> {
    if db::check_refresh(&db)
        .await
        .expect("Couldn't check the token refresh")
    {
        let new_token = refresh_token(spotify_token.refresh_token.clone())
            .await
            .expect("Should be able to handle refresh");
        db::handle_refresh_token(&db, new_token.access_token)
            .await
            .expect("Should be able to update the token");
    }
    let url = "https://api.spotify.com/v1/users/np33/playlists";

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .bearer_auth(spotify_token.access_token.clone())
        .send()
        .await?
        .json::<SpotifyAllPlaylistsRes>()
        .await?;

    let mut playlists: Vec<Playlist> = vec![];
    for playlist in res.items.iter() {
        playlists.push(Playlist {
            name: playlist.name.clone(),
            owner: playlist.owner.display_name.clone(),
            id: playlist.id.clone(),
        })
    }

    Ok(playlists)
}

async fn get_playlist(token: SpotifyAccessToken, id: String) -> Result<Vec<Song>, anyhow::Error> {
    let url = format!("https://api.spotify.com/v1/playlists/{}", id);

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .bearer_auth(token.access_token)
        .query(&[
            ("market", "US"),
            (
                "fields",
                "tracks.items(track(name, artists(name), album(name)))",
            ),
        ])
        .send()
        .await?
        .json::<SpotifyPlaylistRes>()
        .await?;

    let mut songs: Vec<Song> = vec![];
    for song in res.tracks.items.iter() {
        songs.push(Song {
            name: song.track.name.clone(),
            album: song.track.album.name.clone(),
            artist: song.track.artists[0].name.clone(),
        })
    }
    Ok(songs)
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
            println!("You have a token already.")
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
            token.access_token = refreshed_response_token.access_token;
        }
    }
    // if let Some(token) = access_token.clone() {
    //     db::insert_token(&db, token.clone())
    //         .await
    //         .expect("The token should be inserted in the db");
    // }
    // get_playlist(access_token.clone(), "6TJf8ZqOqqRzN2GIldqT4g".to_string())
    //     .await
    //     .unwrap();
    loop {
        match get_command() {
            Command::Search => {
                if let Some(token) = access_token.clone() {
                    let playlists = get_all_playlists(&db, token.clone())
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
                        println!("{} | {} | {}", song.name, song.artist, song.album)
                    }
                }
            }
            Command::View => {
                if let Some(token) = access_token.clone() {
                    let playlists = get_all_playlists(&db, token)
                        .await
                        .expect("There should be playlists to return");
                    for playlist in playlists {
                        println!("{} | {}", playlist.name, playlist.owner)
                    }
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
