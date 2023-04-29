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
use tokio::sync::mpsc::{self, Sender};
use url_builder::URLBuilder;

enum Command {
    Search = 1,
    View,
    Quit,
}

#[derive(Deserialize)]
pub struct SpotifyAuthInfo {
    pub code: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpotifyAccessToken {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
    pub expires_in: i64,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct SpotifyRefreshToken {
    pub access_token: String,
    pub token_type: String,
    pub kscope: String,
    pub expires_in: i64,
}

struct AppState {
    tx: Sender<SpotifyAccessToken>,
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
async fn spotify_auth(
    app_code: web::Query<SpotifyAuthInfo>,
    app_data: web::Data<AppState>,
) -> impl Responder {
    dotenv().unwrap();

    let spotify_token = env::var("SPOTIFY_ACCESS_TOKEN").expect("You need a spotify token");
    let spotify_secret = env::var("SPOTIFY_SECRET").expect("You need a spotify secret");

    let redirect_uri = "http://localhost:8888/callback/spotify";

    let to_encode = format!("{}:{}", spotify_token, spotify_secret);

    let mut b64 = String::new();

    general_purpose::STANDARD.encode_string(to_encode.as_bytes(), &mut b64);

    let client = reqwest::Client::new();
    let params = [
        ("code", app_code.code.as_str()),
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
        .expect("the access token request to send")
        .json::<SpotifyAccessToken>()
        .await
        .expect("the access token response to decode");

    println!("res: {res:?}");

    app_data.tx.send(res.clone()).await.unwrap();
    format!("token: {}", res.access_token.clone())
}

async fn gsat() -> Result<SpotifyAccessToken, anyhow::Error> {
    dotenv()?;

    let redirect_uri = "http://localhost:8888/callback/spotify";

    let scope = "playlist-read-private playlist-read-collaborative";

    let spotify_token = env::var("SPOTIFY_ACCESS_TOKEN").expect("You need a spotify token");

    let (tx, mut rx) = mpsc::channel::<SpotifyAccessToken>(8);

    tokio::spawn(async {
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(AppState { tx: tx.clone() }))
                .service(spotify_auth)
        })
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

    webbrowser::open(ub.build().as_str())?;

    let new_token = rx.recv().await.unwrap();
    Ok(new_token)
}

/*async fn refresh_token(refresh_token: String) -> Result<SpotifyRefreshToken, anyhow::Error> {
    let url = "https://accounts.spotify.com/api/token";

    let spotify_token = env::var("SPOTIFY_ACCESS_TOKEN").expect("You need a spotify token");
    let spotify_secret = env::var("SPOTIFY_SECRET").expect("You need a spotify secret");

    let to_encode = format!("{}:{}", spotify_token, spotify_secret);

    let mut b64 = String::new();

    general_purpose::STANDARD.encode_string(to_encode.as_bytes(), &mut b64);

    let client = reqwest::Client::new();
    let params = [
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token".to_string()),
    ];
    let res = client
        .post(url)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(AUTHORIZATION, format!("Basic {}", b64))
        .form(&params)
        .send()
        .await
        .unwrap()
        .json::<SpotifyRefreshToken>()
        .await
        .unwrap();
    Ok(res)
}*/

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

    for playlist in res.items.iter() {
        println!(
            "{} | {} | {}",
            playlist.name, playlist.owner.display_name, playlist.id
        );
    }

    Ok("playlists worked".to_string())
}

/*async fn get_playlist(token: String, id: String) -> Result<String, anyhow::Error> {
    let url = format!("https://api.spotify.com/v1/playlists/{}", id);

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
}*/

#[tokio::main]
async fn main() {
    let access_token = gsat().await.unwrap();
    get_all_playlists(access_token.access_token.clone())
        .await
        .unwrap();
    // get_playlist(access_token.clone(), "6TJf8ZqOqqRzN2GIldqT4g".to_string())
    //     .await
    //     .unwrap();
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
