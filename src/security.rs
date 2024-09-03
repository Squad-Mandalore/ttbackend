use rand::{distributions::Alphanumeric, Rng};
use sha2::{digest::generic_array::GenericArray, Digest, Sha256};
use sqlx::PgPool;

fn hash_password() {
}

// creates a random string and saves it to the db
fn create_salt(pool: PgPool, employee_id: i32, length: usize) {
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
}

async fn get_salt(pool: &PgPool, employee_id: i32) {
    let salt = sqlx::query!(
        r#"SELECT pw_salt FROM employee WHERE employee_id = $1"#,
        employee_id
    )
    .fetch_one(pool)
    .await;
}

// takes a string and applies it to the current password
fn apply_spice(current_password: String, spice: String) -> String {
    format!("{}{}", current_password, spice)
}

// takes the current password and iterate the hasing function over it x times
fn iterate_hash(current_password: String, x: i32) -> Vec<u8> {
let mut hasher = Sha256::new();
    hasher.update(current_password);
    let mut result = hasher.finalize_reset();

    for _ in 1..x {
        hasher.update(&result);
        result = hasher.finalize_reset();
    }

    result.into_iter().collect()
}

// takes a string and a employee_id and calls the hash_password function and compares it with the current employee password
fn verify_password() {
}
