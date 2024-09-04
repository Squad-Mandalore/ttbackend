use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use axum::{
    middleware,
    response::{self, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use tower_http::services::ServeFile;
use ttbackend::{
    auth::{auth, login, refresh}, database::set_up_database, graphql::{create_schema, graphql_handler}, pdf, shutdown_signal, tracing_setup::{remove_old_logfiles, setup_tracing}
};

#[cfg(debug_assertions)]
async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[tokio::main]
async fn main() {

    pdf::generate_pdf("2024-01", 1, "Markus", "Quarkus", "markus@quarkus.nvim").await;

    

    // setup tracing
    let _guard = setup_tracing();
    let _ = remove_old_logfiles().await;

    // setup database connection pool
    let database_pool = set_up_database().await;
    let schema = create_schema(database_pool.clone());

    // build our application with a single route
    let app = Router::new()
        .route(
            "/graphql",
            post(graphql_handler).layer(middleware::from_fn(auth)),
        )
        .layer(Extension(schema))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .with_state(database_pool);

    #[cfg(debug_assertions)]
    let debug = Router::new()
        .route("/playground", get(graphql_playground))
        .route_service("/docs", ServeFile::new("docs.html"))
        .route_service("/api.yaml", ServeFile::new("api.yaml"));

    #[cfg(debug_assertions)]
    let app = app.nest("/debug", debug);

    // run our app with hyper, listening globally on port 3001*
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    
}
