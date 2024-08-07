use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use ttbackend::{
    database::set_up_connection_pool, graphql::create_schema, tracing_setup::{remove_old_logfiles, setup_tracing}
};

async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[tokio::main]
async fn main() {
    // setup tracing
    let _guard = setup_tracing();
    let _ = remove_old_logfiles().await;

    // setup database connection pool
    let database_pool = set_up_connection_pool().await;

    // build our application with a single route
    let app = Router::new().route(
        "/",
        get(graphql_playground).post_service(GraphQL::new(create_schema(database_pool))),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
