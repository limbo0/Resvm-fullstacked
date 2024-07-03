pub mod app;
pub mod crud_properties;
pub mod error_template;
pub mod errors;
pub mod models;
pub mod schema;
use argon2::{self, Config};
use axum::extract::FromRef;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use dotenvy::dotenv;
use leptos::LeptosOptions;
use rand::RngCore;
use std::{env, sync::Arc};

type SharedPooledConnection = Arc<Pool<ConnectionManager<PgConnection>>>;

pub fn get_connection_pool() -> SharedPooledConnection {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(url);

    Arc::new(
        Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Could not build connection pool"),
    )
}

/// Derive FromRef to allow multiple items in state, using Axum’s
/// SubStates pattern.
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: SharedPooledConnection,
}

/// Password hasher.
pub async fn salt_password(secret: String) -> anyhow::Result<String> {
    let mut salt = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut salt);
    // println!("salt: {:?}", salt);

    let config = Config::default();
    let hash_p = argon2::hash_encoded(secret.as_bytes(), &salt, &config).unwrap();
    Ok(hash_p)
}
