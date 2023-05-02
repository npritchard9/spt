use super::{Playlist, Song, SpotifyAccessToken};
use playlist::models::{
    all_playlists::SpotifyAllPlaylistsRes, currently_playing::SpotifyCurrentlyPlayingRes,
    playlist::SpotifyPlaylistRes, search::SpotifySearchRes,
};
use reqwest::header::CONTENT_LENGTH;

pub async fn get_all_playlists(
    spotify_token: SpotifyAccessToken,
) -> Result<Vec<Playlist>, anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/playlists";

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

pub async fn get_currently_playing(
    spotify_token: SpotifyAccessToken,
) -> Result<Song, anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/currently-playing";

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .bearer_auth(spotify_token.access_token.clone())
        .query(&[("market", "US")])
        .send()
        .await?
        .json::<SpotifyCurrentlyPlayingRes>()
        .await?;

    let song: Song = Song {
        name: res.item.name,
        album: res.item.album.name,
        artist: res.item.artists[0].name.clone(),
    };

    Ok(song)
}

pub async fn skip_to_next(spotify_token: SpotifyAccessToken) -> Result<(), anyhow::Error> {
    let url = "https://api.spotify.com/v1/me/player/next";

    let client = reqwest::Client::new();
    client
        .post(url)
        .bearer_auth(spotify_token.access_token.clone())
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
        .bearer_auth(spotify_token.access_token.clone())
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
        .bearer_auth(spotify_token.access_token.clone())
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
    for song in res.tracks.items.iter() {
        songs.push(Song {
            name: song.name.clone(),
            album: song.album.name.clone(),
            artist: song.artists[0].name.clone(),
        })
    }

    Ok(songs)
}
