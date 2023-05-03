use super::{Playlist, Song, SpotifyAccessToken};
use playlist::models::{
    all_playlists::SpotifyAllPlaylistsRes, currently_playing::SpotifyCurrentlyPlayingRes,
    playlist::SpotifyPlaylistRes, search::SpotifySearchRes,
};
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SpotifyJSON {
    uris: Vec<String>,
}

pub async fn get_all_playlists(
    spotify_token: SpotifyAccessToken,
) -> Result<Vec<Playlist>, anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/playlists";

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .bearer_auth(spotify_token.access_token)
        .send()
        .await?
        .json::<SpotifyAllPlaylistsRes>()
        .await?;

    let mut playlists: Vec<Playlist> = vec![];
    for playlist in res.items {
        playlists.push(Playlist {
            name: playlist.name,
            owner: playlist.owner.display_name,
            id: playlist.id,
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
    for song in res.tracks.items {
        songs.push(Song {
            name: song.track.name,
            album: song.track.album.name,
            artist: song.track.artists[0].name.clone(),
            uri: "".to_string(),
        })
    }
    Ok(songs)
}

pub async fn get_currently_playing(
    spotify_token: SpotifyAccessToken,
) -> Result<Song, anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/currently-playing";

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .bearer_auth(spotify_token.access_token)
        .query(&[("market", "US")])
        .send()
        .await?
        .json::<SpotifyCurrentlyPlayingRes>()
        .await?;

    let song: Song = Song {
        name: res.item.name,
        album: res.item.album.name,
        artist: res.item.artists[0].name.clone(),
        uri: "".to_string(),
    };

    Ok(song)
}

pub async fn skip_to_next(spotify_token: SpotifyAccessToken) -> Result<(), anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/next";

    let client = reqwest::Client::new();
    client
        .post(url)
        .bearer_auth(spotify_token.access_token)
        .header(CONTENT_LENGTH, 0)
        .send()
        .await?;

    Ok(())
}

pub async fn skip_to_prev(spotify_token: SpotifyAccessToken) -> Result<(), anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/previous";

    let client = reqwest::Client::new();
    client
        .post(url)
        .bearer_auth(spotify_token.access_token)
        .header(CONTENT_LENGTH, 0)
        .send()
        .await?;

    Ok(())
}

pub async fn search_for_item(
    spotify_token: SpotifyAccessToken,
    q: &str,
) -> Result<Vec<Song>, anyhow::Error> {
    let url = "https://api.spotify.com/v1/search";

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .bearer_auth(spotify_token.access_token)
        .query(&[
            ("q", q),
            ("market", "US"),
            ("type", "track"),
            ("limit", "5"),
        ])
        .send()
        .await?
        .json::<SpotifySearchRes>()
        .await?;

    let mut songs: Vec<Song> = vec![];
    for song in res.tracks.items {
        songs.push(Song {
            name: song.name,
            album: song.album.name,
            artist: song.artists[0].name.clone(),
            uri: song.uri,
        })
    }

    Ok(songs)
}

pub async fn add_to_queue(
    spotify_token: SpotifyAccessToken,
    uri: String,
) -> Result<(), anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/queue";

    let client = reqwest::Client::new();
    client
        .post(url)
        .bearer_auth(spotify_token.access_token)
        .header(CONTENT_LENGTH, 0)
        .query(&[("uri", uri)])
        .send()
        .await?;

    Ok(())
}

pub async fn pause(spotify_token: SpotifyAccessToken) -> Result<(), anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/pause";

    let client = reqwest::Client::new();
    client
        .put(url)
        .bearer_auth(spotify_token.access_token)
        .header(CONTENT_LENGTH, 0)
        .send()
        .await?;

    Ok(())
}

pub async fn resume(spotify_token: SpotifyAccessToken) -> Result<(), anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/play";

    let client = reqwest::Client::new();
    client
        .put(url)
        .bearer_auth(spotify_token.access_token)
        .header(CONTENT_LENGTH, 0)
        .send()
        .await?;

    Ok(())
}

pub async fn start_playing(
    spotify_token: SpotifyAccessToken,
    uris: Vec<String>,
) -> Result<(), anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/play";
    let json = SpotifyJSON { uris };

    let client = reqwest::Client::new();
    client
        .put(url)
        .bearer_auth(spotify_token.access_token)
        .header(CONTENT_TYPE, "application/json")
        .json(&json)
        .send()
        .await?;

    Ok(())
}

pub async fn shuffle(
    spotify_token: SpotifyAccessToken,
    shuffle_state: bool,
) -> Result<(), anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/shuffle";

    let client = reqwest::Client::new();
    client
        .put(url)
        .bearer_auth(spotify_token.access_token)
        .header(CONTENT_LENGTH, 0)
        .query(&[("state", shuffle_state)])
        .send()
        .await?;

    Ok(())
}

pub async fn add_to_playlist(
    spotify_token: SpotifyAccessToken,
    pid: String,
    uris: Vec<String>,
) -> Result<(), anyhow::Error> {
    let url = format!("https://api.spotify.com/v1/playlists/{}/tracks", pid);
    let json = SpotifyJSON { uris };

    let client = reqwest::Client::new();
    client
        .post(url)
        .bearer_auth(spotify_token.access_token)
        .header(CONTENT_TYPE, "application/json")
        .json(&json)
        .send()
        .await?
        .text()
        .await?;

    Ok(())
}
