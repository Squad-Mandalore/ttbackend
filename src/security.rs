use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

pub async fn hash_password(
    pool: &PgPool,
    employee_id: &i32,
    given_password: String,
) -> sqlx::Result<String> {
    let salt = get_salt(pool, employee_id).await?;
    let pepper = dotenvy::var("PEPPER").expect("PEPPER variable not set.");
    let keychain_number = dotenvy::var("KEYCHAIN_NUMBER")
        .expect("KEYCHAIN_NUMBER variable not set.")
        .parse::<i32>()
        .expect("KEYCHAIN_NUMBER is not a proper number.");

    match salt {
        // first login
        None => Ok(given_password),
        // proceed as normal
        Some(some_salt) => {
            let spiced_password = format!("{}{}{}", given_password, some_salt, pepper);
            Ok(iterate_hash(spiced_password, keychain_number))
        }
    }
}

// creates a random string and saves it to the db
pub async fn create_salt(pool: &PgPool, employee_id: &i32, length: usize) -> sqlx::Result<()> {
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();

    let _ = sqlx::query!(
        "UPDATE employee SET pw_salt = $2 WHERE employee_id = $1",
        employee_id,
        random_string,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// fetches the salt from the database and will return None if there is no salt
// e.g. on the initial login
async fn get_salt(pool: &PgPool, employee_id: &i32) -> sqlx::Result<Option<String>> {
    let result = sqlx::query!(
        "SELECT pw_salt FROM employee WHERE employee_id = $1",
        employee_id
    )
    .fetch_optional(pool)
    .await?
    .and_then(|record| record.pw_salt);

    Ok(result)
}

// takes the current password and iterate the hasing function over it x times
fn iterate_hash(given_password: String, x: i32) -> String {
    // hash it x times
    let mut hasher = Sha256::new();
    hasher.update(given_password);
    let mut result = hasher.finalize_reset();

    for _ in 1..x {
        hasher.update(&result);
        result = hasher.finalize_reset();
    }

    // convert bytes into hexadecimal string
    result.iter().map(|byte| format!("{:02x}", byte)).collect()
}

// takes a string and a employee_id and calls the hash_password function and compares it with the current employee password
pub async fn verify_password(
    pool: &PgPool,
    employee_id: &i32,
    given_password: String,
) -> sqlx::Result<bool> {
    let hashed_password = hash_password(pool, employee_id, given_password).await?;
    let current_password = get_password(pool, employee_id).await?;

    Ok(hashed_password == current_password)
}

async fn get_password(pool: &PgPool, employee_id: &i32) -> sqlx::Result<String> {
    sqlx::query!(
        "SELECT password FROM employee WHERE employee_id = $1",
        employee_id
    )
    .fetch_one(pool)
    .await
    .and_then(|record| Ok(record.password))
}
