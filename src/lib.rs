pub mod app;
pub mod crud_properties;
pub mod error_template;
pub mod errors;
pub mod models;
pub mod schema;
use axum::extract::FromRef;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use dotenvy::dotenv;
use leptos::LeptosOptions;
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

// pub mod ssr {
//     // use http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode};
//     use diesel::pg::PgConnection;
//     use diesel::prelude::*;
//     use dotenvy::dotenv;
//     use std::env;
//
//     pub fn establish_connection() -> PgConnection {
//         dotenv().ok();
//
//         let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//         PgConnection::establish(&database_url)
//             .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
//     }
// }
