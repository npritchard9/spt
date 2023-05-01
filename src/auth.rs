use actix_web::{get, web, App, HttpServer, Responder};
use base64::{engine::general_purpose, Engine as _};
use dotenvy::dotenv;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::env;
use tokio::sync::mpsc::{self, Sender};
use url_builder::URLBuilder;

pub struct AppState {
    pub tx: Sender<SpotifyAccessToken>,
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
    pub scope: String,
    pub expires_in: i64,
}

#[get("/callback/spotify")]
pub async fn spotify_auth(
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

pub async fn gsat() -> Result<SpotifyAccessToken, anyhow::Error> {
    dotenv()?;

    let redirect_uri = "http://localhost:8888/callback/spotify";

    let scope = "playlist-read-private playlist-read-collaborative user-read-currently-playing user-modify-playback-state";

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

pub async fn refresh_token(refresh_token: String) -> Result<SpotifyRefreshToken, anyhow::Error> {
    dotenv()?;
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
}
