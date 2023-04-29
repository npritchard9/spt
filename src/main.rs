use actix_web::{get, web, App, HttpServer, Responder};
use anyhow::anyhow;
use base64::{engine::general_purpose, Engine as _};
use dotenvy::dotenv;
use playlist::models::{all_playlists::SpotifyAllPlaylistsRes, playlist::SpotifyPlaylistRes};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::{
    env,
    io::{stdin, stdout, Write},
    str::FromStr,
};
use url_builder::URLBuilder;

enum Command {
    Search = 1,
    View,
    Quit,
}

#[derive(Deserialize)]
struct SpotifyAuthInfo {
    code: String,
}

#[derive(Debug, Deserialize)]
struct SpotifyAccessTokenRes {
    access_token: String,
    token_type: String,
    scope: String,
    expires_in: i64,
    refresh_token: String,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Command::Search),
            "2" => Ok(Command::View),
            "3" => Ok(Command::Quit),
            _ => Err(anyhow!("Invalid command.")),
        }
    }
}

fn get_command() -> Command {
    let mut input = String::new();
    println!("What do you want to do?");
    println!("1. Search for a song.");
    println!("2. View your playlist.");
    println!("3. Quit.");
    stdout().flush().unwrap();
    stdin().read_line(&mut input).unwrap();
    let command = input.trim().parse::<Command>().unwrap();
    command
}

#[get("/callback/spotify")]
async fn spotify_auth(info: web::Query<SpotifyAuthInfo>) -> impl Responder {
    dotenv().unwrap();

    let spotify_token = env::var("SPOTIFY_ACCESS_TOKEN").expect("You need a spotify token");
    let spotify_secret = env::var("SPOTIFY_SECRET").expect("You need a spotify secret");

    let redirect_uri = "http://localhost:8888/callback/spotify";

    let to_encode = format!("{}:{}", spotify_token, spotify_secret);

    let mut b64 = String::new();

    general_purpose::STANDARD.encode_string(to_encode.as_bytes(), &mut b64);

    let client = reqwest::Client::new();
    let params = [
        ("code", info.code.as_str()),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];
    let res = client
        .post("https://accounts.spotify.com/api/token")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(AUTHORIZATION, format!("Basic {}", b64))
        .form(&params)
        .send()
        .await
        .unwrap()
        .json::<SpotifyAccessTokenRes>()
        .await
        .unwrap();

    format!("token: {}", res.access_token)
}

async fn gsat() -> Result<(), anyhow::Error> {
    dotenv()?;

    let redirect_uri = "http://localhost:8888/callback/spotify";

    let scope = "playlist-read-private playlist-read-collaborative";

    let spotify_token = env::var("SPOTIFY_ACCESS_TOKEN").expect("You need a spotify token");

    tokio::spawn(async {
        let server = HttpServer::new(move || App::new().service(spotify_auth))
            .bind(("127.0.0.1", 8888))
            .unwrap()
            .run();
        server.await.unwrap();
    });

    let mut ub = URLBuilder::new();
    ub.set_protocol("https")
        .set_host("accounts.spotify.com/authorize")
        .add_param("response_type", "code")
        .add_param("scope", scope)
        .add_param("client_id", &spotify_token.to_string())
        .add_param("redirect_uri", redirect_uri);

    if webbrowser::open(ub.build().as_str()).is_ok() {
        println!("ok")
    }

    Ok(())
}

async fn get_all_playlists(token: String) -> Result<String, anyhow::Error> {
    let url = "https://api.spotify.com/v1/users/np33/playlists";

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await?
        .json::<SpotifyAllPlaylistsRes>()
        .await?;

    println!("{res:#?}");

    Ok("".to_string())
}

async fn get_playlist(token: String) -> Result<String, anyhow::Error> {
    let url = "https://api.spotify.com/v1/playlists/6TJf8ZqOqqRzN2GIldqT4g";

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .bearer_auth(token)
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

    for song in res.tracks.items.iter() {
        println!(
            "{} | {} | {}",
            song.track.name, song.track.album.name, song.track.artists[0].name
        );
    }
    Ok("".to_string())
}

#[tokio::main]
async fn main() {
    gsat().await.unwrap();
    // get_all_playlists(access_token.clone()).await.unwrap();
    // get_playlist(access_token.clone()).await.unwrap();
    loop {
        match get_command() {
            Command::Search => {
                println!("search")
            }
            Command::View => println!("view"),
            Command::Quit => break,
        };
    }
}
