use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ttbackend::graphql::create_schema;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool, Row, FromRow};


const DB_URL: &str = "sqlite://squadb.db";

//struct machen damit query_as funktioniert
#[derive(Clone, FromRow, Debug)]
struct User {
    id: i64,
    name: String,
}

async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success!"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Datbase already exists");
    }

    //establish connection to the database
    let db = SqlitePool::connect(DB_URL).await.unwrap();

    //so kann man migration bums machen
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let migrations = std::path::Path::new(&crate_dir).join("./migrations");

    let migration_results = sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(&db)
        .await;

    match migration_results {
        Ok(_) => println!("Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }

    println!("migration: {:?}", migration_results);


    //query schema to list all tables
    let result = sqlx::query(
        "SELECT name
         FROM sqlite_schema
         WHERE type ='table'
         AND name NOT LIKE 'sqlite_%';",
    )
    .fetch_all(&db)
    .await
    .unwrap();

    for (idx, row) in result.iter().enumerate() {
        println!("[{}]: {:?}", idx, row.get::<String, &str>("name"));
    }

    //erstmal einen krassen dude zur db hinzufuegen
    let result = sqlx::query("INSERT INTO users (name) VALUES (?)")
        .bind("Markus Quarkus")
        .execute(&db)
        .await
        .unwrap();

    println!("Query result: {:?}", result);

    //und jetzt kieken wa mal was fuer krasse dudes in der db sind
    let user_results = sqlx::query_as::<_,User>("SELECT id, name FROM users")
        .fetch_all(&db)
        .await
        .unwrap();
    for user in user_results {
        println!("[{}] name: {}", user.id, &user.name);
    }

    //jetzt muessen wir leider den krassen dude aus der db loeschen for the sake of the example
    let delete_results = sqlx::query("DELETE FROM users WHERE name=$1")
        .bind("Markus Quarkus")
        .execute(&db)
        .await
        .unwrap();

    println!("Query result: {:?}", delete_results);

    //funktion gehen auch
    get_all_users(db);

    // build our application with a single route
    let app = Router::new().route(
        "/",
        get(graphql_playground).post_service(GraphQL::new(create_schema())),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn get_all_users(db: SqlitePool) {

    let user_results = sqlx::query_as::<_,User>("SELECT id, name FROM users")
        .fetch_all(&db)
        .await
        .unwrap();
    for user in user_results {
        println!("[{}] name: {}", user.id, &user.name);
    }
}
