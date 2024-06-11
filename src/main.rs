use axum::{
    body::Body,
    extract::{FromRef, Path, State},
    http::Request,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use resvm::{app::ResvmApp, get_connection_pool, AppState};

//Define a handler to test extractor with state
async fn custom_handler(
    Path(id): Path<String>,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> Response {
    let handler = leptos_axum::render_app_to_stream_with_context(
        options,
        move || {
            provide_context(id.clone());
        },
        ResvmApp,
    );
    handler(req).await.into_response()
}

#[tokio::main]
async fn main() {
    // Setting this to None means we'll be using cargo-leptos and its env vars
    let conf = get_configuration(Some("/home/khewa/resv_manager/resvm/Cargo.toml"))
        .await
        .unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(ResvmApp);

    let app_state = AppState {
        leptos_options,
        pool: get_connection_pool(),
    };

    // build our application with a route
    let app = Router::new()
        .route("/something", get(custom_handler))
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || provide_context(app_state.clone())
            },
            || view! { <ResvmApp/> },
        )
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
