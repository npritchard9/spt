use actix_web::{get, web, App, HttpServer, Responder};
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use tokio::sync::mpsc::{self, Sender};
use url_builder::URLBuilder;

pub struct AppState {
    pub tx: Sender<SpotifyAccessToken>,
    pub id: String,
    pub secret: String,
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
    let spotify_id = app_data.id.clone();
    let spotify_secret = app_data.secret.clone();

    let redirect_uri = "http://localhost:8888/callback/spotify";

    let to_encode = format!("{}:{}", spotify_id, spotify_secret);

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

    app_data.tx.send(res.clone()).await.unwrap();
    format!("token: {}", res.access_token.clone())
}

pub async fn gsat(
    spotify_id: String,
    spotify_secret: String,
) -> Result<SpotifyAccessToken, anyhow::Error> {
    let redirect_uri = "http://localhost:8888/callback/spotify";

    let scope = "playlist-read-private playlist-read-collaborative user-read-currently-playing user-modify-playback-state";

    let (tx, mut rx) = mpsc::channel::<SpotifyAccessToken>(8);

    let id = spotify_id.clone();

    tokio::spawn(async {
        let server = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(AppState {
                    tx: tx.clone(),
                    id: id.clone(),
                    secret: spotify_secret.clone(),
                }))
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
        .add_param("client_id", spotify_id.clone().as_str())
        .add_param("redirect_uri", redirect_uri);

    webbrowser::open(ub.build().as_str())?;

    let new_token = rx.recv().await.unwrap();
    Ok(new_token)
}

pub async fn refresh_token(refresh_token: String, spotify_id: String, spotify_secret: String) -> Result<SpotifyRefreshToken, anyhow::Error> {
    let url = "https://accounts.spotify.com/api/token";

    let to_encode = format!("{}:{}", spotify_id, spotify_secret);

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
