use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use surrealdb::engine::local::{Db, SpeeDb};
use surrealdb::Surreal;

use crate::SpotifyAccessToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBToken {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub scope: String,
    pub expires_in: i64,
    pub time: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCredentials {
    pub client_id: String,
    pub secret: String,
}

pub async fn insert_client_credentials(
    db: &Surreal<Db>,
    creds: ClientCredentials,
) -> surrealdb::Result<()> {
    let existing_creds: Option<ClientCredentials> = db.select(("app", "creds")).await?;
    if existing_creds.is_some() {
        let _creds: Option<String> = db
            .delete(("app", "creds"))
            .await
            .expect("should be able to delete creds");
    }
    let _creds: Option<ClientCredentials> = db
        .create(("app", "creds"))
        .content(creds)
        .await
        .expect("should be able to create creds");
    Ok(())
}

pub async fn select_credentials(db: &Surreal<Db>) -> surrealdb::Result<Option<ClientCredentials>> {
    let creds: Option<ClientCredentials> = db.select(("app", "creds")).await?;
    Ok(creds)
}

pub async fn delete_credentials(db: &Surreal<Db>) -> surrealdb::Result<()> {
    let _creds: Vec<ClientCredentials> = db.delete("app").await?;
    Ok(())
}

pub async fn insert_token(
    db: &Surreal<Db>,
    old_token: SpotifyAccessToken,
) -> surrealdb::Result<()> {
    let _token: Option<DBToken> = db
        .create(("token", "noah"))
        .content(DBToken {
            access_token: old_token.access_token,
            refresh_token: old_token.refresh_token,
            time: SystemTime::now(),
            token_type: old_token.token_type,
            scope: old_token.scope,
            expires_in: old_token.expires_in,
        })
        .await?;
    Ok(())
}

pub async fn delete_token(db: &Surreal<Db>) -> surrealdb::Result<()> {
    let _token: Vec<DBToken> = db.delete("token").await?;
    Ok(())
}

pub async fn select_token(db: &Surreal<Db>) -> surrealdb::Result<Option<SpotifyAccessToken>> {
    let sql = "SELECT access_token, refresh_token, token_type, scope, expires_in FROM type::table($table);";
    let mut result = db.query(sql).bind(("table", "token")).await?;
    let token: Option<SpotifyAccessToken> = result.take(0)?;
    Ok(token)
}

pub async fn check_refresh(db: &Surreal<Db>) -> surrealdb::Result<bool> {
    match db.select(("token", "noah")).await {
        Ok(token) => {
            let token: DBToken = token.expect("Token must exist to refresh");
            let curr = SystemTime::now();
            let elapsed = curr.duration_since(token.time).unwrap();
            Ok(elapsed > Duration::new(token.expires_in as u64, 0))
        }
        Err(e) => Err(e),
    }
}

pub async fn update_token(db: &Surreal<Db>, new_access_token: String) -> surrealdb::Result<()> {
    let old_token: DBToken = db
        .select(("token", "noah"))
        .await?
        .expect("Must be an old token to update");
    let _new_token: Option<DBToken> = db
        .update(("token", "noah"))
        .content(DBToken {
            access_token: new_access_token,
            refresh_token: old_token.refresh_token,
            time: SystemTime::now(),
            token_type: old_token.token_type,
            scope: old_token.scope,
            expires_in: old_token.expires_in,
        })
        .await?;
    println!("updated token");
    Ok(())
}

pub async fn get_db() -> surrealdb::Result<Surreal<Db>> {
    let db = Surreal::new::<SpeeDb>("/home/noah/.surrealdb/data/spotify.db").await?;
    db.use_ns("my_ns").use_db("my_db").await?;

    Ok(db)
}
