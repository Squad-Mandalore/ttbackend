use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use axum::{
    middleware,
    response::{self, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::services::ServeFile;
use ttbackend::{
    auth::{auth, login, refresh},
    database::set_up_database,
    graphql::{create_schema, graphql_handler},
    shutdown_signal,
    tracing_setup::{remove_old_logfiles, setup_tracing},
};

#[cfg(debug_assertions)]
async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[tokio::main]
async fn main() {
    // setup tracing
    let _guard = setup_tracing();
    let _ = remove_old_logfiles().await;

    // setup database connection pool
    let database_pool = set_up_database().await;
    let app = app(database_pool);

    #[cfg(debug_assertions)]
    let debug = Router::new()
        .route("/playground", get(graphql_playground))
        .route_service("/docs", ServeFile::new("docs.html"))
        .route_service("/api.yaml", ServeFile::new("api.yaml"));

    #[cfg(debug_assertions)]
    let app = app.nest("/debug", debug);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn app(database_pool: PgPool) -> Router {
    let schema = create_schema(database_pool.clone());

    // build our application with a single route
    Router::new()
        .route(
            "/graphql",
            post(graphql_handler).layer(middleware::from_fn(auth)),
        )
        .layer(Extension(schema))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .with_state(database_pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::{Service, ServiceExt};

    #[tokio::test]
    async fn login_test() {
        let database_pool = set_up_database().await;
        let app = app(database_pool);

        let json_payload = json!({
        "email": "mace.windu@deepcore.com",
        "password": "jedi456"
        });

        // Convert your JSON payload to a string
        let json_string = serde_json::to_string(&json_payload).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json_string))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Print the status code
        println!("Status: {}", response.status());

        // Extract the body from the response
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();

        // Convert the body bytes to a UTF-8 string and print
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
        println!("Body: {}", body_string);

        assert_eq!(1, 2)
    }
}
