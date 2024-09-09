use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use axum::{
    middleware,
    response::{self, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::{cors, services::ServeFile};
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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

fn app(database_pool: PgPool) -> Router {
    let schema = create_schema(database_pool.clone());

    let cors = cors::CorsLayer::new()
        // allow `POST` when accessing the resource
        .allow_methods([hyper::Method::POST])
        .allow_headers([http::header::AUTHORIZATION, http::header::CONTENT_TYPE])
        // allow requests from any origin
        .allow_origin(cors::Any);

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
        .layer(cors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use hyper::StatusCode;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tower::ServiceExt;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Authorization {
        access_token: String,
        refresh_token: String,
    }

    async fn login() -> (Router, Authorization) {
        let database_pool = set_up_database().await;
        let app = app(database_pool);

        let json_payload = json!({
        "email": "mace.windu@deepcore.com",
        "password": "mace.windu@deepcore.com"
        });

        // Convert your JSON payload to a string
        let json_string = serde_json::to_string(&json_payload).unwrap();

        let response = app
            .clone()
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
        assert_eq!(response.status(), StatusCode::OK);

        // Extract the body from the response
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();

        // Convert the body bytes to a UTF-8 string and print
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
        println!("Body: {}", body_string);

        let authorization: Authorization =
            serde_json::from_str(&body_string).expect("Json not correct");
        (app, authorization)
    }

    #[tokio::test]
    async fn test_claims() {
        let (app, claims) = login().await;

        let json_payload = json!({
            "query":"query {\n  timers {\n    employeeId\n  }\n}"
        });

        let json_string = serde_json::to_string(&json_payload).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/graphql")
                    .header("Content-Type", "application/json")
                    .header("authorization", format!("Bearer {}", claims.access_token))
                    .body(Body::from(json_string))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Extract the body from the response
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();

        // Convert the body bytes to a UTF-8 string and print
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
        println!("Body timers: {}", body_string);
    }

    #[tokio::test]
    async fn test_refresh() {
        let (app, claims) = login().await;

        let json_payload = json!({
            "refreshToken":format!("{}", claims.refresh_token)
        });

        let json_string = serde_json::to_string(&json_payload).unwrap();
        println!("json string: {}", json_string);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/refresh")
                    .header("Content-Type", "application/json")
                    .body(Body::from(json_string))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Extract the body from the response
        let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();

        // Convert the body bytes to a UTF-8 string and print
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
        println!("Body timers: {}", body_string);
    }
}
