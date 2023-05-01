use super::{Playlist, Song, SpotifyAccessToken};
use crate::auth::*;
use crate::db;
use playlist::models::{all_playlists::SpotifyAllPlaylistsRes, playlist::SpotifyPlaylistRes};
use surrealdb::{engine::local::Db, Surreal};

// this also checks if we need a refresh
pub async fn get_all_playlists(
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

pub async fn get_playlist(
    token: SpotifyAccessToken,
    id: String,
) -> Result<Vec<Song>, anyhow::Error> {
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
