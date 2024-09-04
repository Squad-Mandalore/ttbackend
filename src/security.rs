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

    match salt {
        // first login
        None => Ok(given_password),
        // proceed as normal
        Some(some_salt) => {
            let spiced_password = format!("{}{}{}", given_password, some_salt, pepper);
            Ok(iterate_hash(spiced_password))
        }
    }
}

// creates a random string and saves it to the db
pub async fn create_salt(pool: &PgPool, employee_id: &i32) -> sqlx::Result<()> {
    let salt_length = dotenvy::var("SALT_LENGTH")
        .expect("SALT_LENGTH variable not set.")
        .parse::<usize>()
        .expect("SALT_LENGTH is not a proper number.");

    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(salt_length)
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
    sqlx::query!(
        "SELECT pw_salt FROM employee WHERE employee_id = $1",
        employee_id
    )
    .fetch_one(pool)
    .await
    .map(|record| record.pw_salt)
}

// takes the current password and iterate the hasing function over it x times
fn iterate_hash(given_password: String) -> String {
    let keychain_number = dotenvy::var("KEYCHAIN_NUMBER")
        .expect("KEYCHAIN_NUMBER variable not set.")
        .parse::<i32>()
        .expect("KEYCHAIN_NUMBER is not a proper number.");

    // hash it x times
    let mut hasher = Sha256::new();
    hasher.update(given_password);
    let mut result = hasher.finalize_reset();

    for _ in 1..keychain_number {
        hasher.update(result);
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
    .await.map(|record| record.password)
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::service::employee::update_password;

    use super::*;

    #[sqlx::test(fixtures(
        "../fixtures/truncate.sql",
        "../fixtures/address.sql",
        "../fixtures/employee.sql",
    ))]
    async fn test_get_some_salt(pool: PgPool) -> sqlx::Result<()> {
        // given is the employee_id
        let employee_id = 1;

        // when get_salt is called
        let salt = get_salt(&pool, &employee_id).await?;

        // then salt should be 'no'
        assert!(salt.is_some());
        assert_eq!(salt.unwrap(), String::from("no"));
        Ok(())
    }

    #[sqlx::test(fixtures(
        "../fixtures/truncate.sql",
        "../fixtures/address.sql",
        "../fixtures/employee.sql",
    ))]
    async fn test_get_none_salt(pool: PgPool) -> sqlx::Result<()> {
        // given is the employee_id
        let employee_id = 2;

        // when get_salt is called
        let salt = get_salt(&pool, &employee_id).await?;

        // then the salt should be none
        assert!(salt.is_none());
        Ok(())
    }

    #[sqlx::test(fixtures(
        "../fixtures/truncate.sql",
        "../fixtures/address.sql",
        "../fixtures/employee.sql",
    ))]
    async fn test_get_password(pool: PgPool) -> sqlx::Result<()> {
        // given is the employee_id
        let employee_id = 2;

        // when get_password is called
        let password = get_password(&pool, &employee_id).await?;

        // then the password should be catgirls123
        assert_eq!(password, String::from("catgirls123"));
        Ok(())
    }

    #[test]
    fn test_iterate_hash_length() {
        // given is a password to get hashed and a KEYCHAIN_NUMBER
        let password = String::from("catgirls123");

        // when iterate_hash is called
        let hashed_password = iterate_hash(password);

        // then the hashed_password should be a String with 64 hexadecimal characters
        assert_eq!(64, hashed_password.len());
    }

    #[test]
    fn test_iterate_hash_iterations() {
        // given is a password to get hashed and a KEYCHAIN_NUMBER and another KEYCHAIN_NUMBER
        let password = String::from("catgirls123");
        const KEYCHAIN_NUMBER: i32 = 361;
        const ANOTHER_KEYCHAIN_NUMBER: i32 = 187;
        env::set_var("KEYCHAIN_NUMBER", KEYCHAIN_NUMBER.to_string());

        // when iterate_hash is called with both keychain_numbers
        let hashed_password = iterate_hash(password.clone());
        env::set_var("KEYCHAIN_NUMBER", ANOTHER_KEYCHAIN_NUMBER.to_string());
        let another_hashed_password = iterate_hash(password);

        // then the both hashed_passwords should not match
        assert_ne!(hashed_password, another_hashed_password);
    }

    #[sqlx::test(fixtures(
        "../fixtures/truncate.sql",
        "../fixtures/address.sql",
        "../fixtures/employee.sql",
    ))]
    async fn test_create_salt(pool: PgPool) -> sqlx::Result<()> {
        // given is the employee_id and the salt_length
        let employee_id = 2;
        const SALT_LENGTH: usize = 64;
        env::set_var("SALT_LENGTH", SALT_LENGTH.to_string());

        // when create_salt is called
        create_salt(&pool, &employee_id).await?;

        // then there should be a salt inside the database with the same length and the SALT_LENGTH
        let salt = get_salt(&pool, &employee_id).await?;
        assert!(salt.is_some());
        assert_eq!(SALT_LENGTH, salt.unwrap().len());

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../fixtures/truncate.sql",
        "../fixtures/address.sql",
        "../fixtures/employee.sql",
    ))]
    async fn test_verify_password_and_hash_password_first_login(pool: PgPool) -> sqlx::Result<()> {
        // given is the employee_id and the password to verify
        let employee_id = 2;
        let given_password = String::from("catgirls123");

        // when verify_password is called
        let check = verify_password(&pool, &employee_id, given_password).await?;

        // then it should match since it is the first login and no salt was already existing
        assert!(check);

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../fixtures/truncate.sql",
        "../fixtures/address.sql",
        "../fixtures/employee.sql",
    ))]
    async fn test_update_password(pool: PgPool) -> sqlx::Result<()> {
        // given is a user which has not logged in yet and its password
        let employee_id = 1;
        let password = get_password(&pool, &employee_id).await?;
        const SALT_LENGTH: usize = 64;
        env::set_var("SALT_LENGTH", SALT_LENGTH.to_string());

        // when password is updated
        update_password(&pool, &employee_id, password).await?;

        // then the password should be hashed and there is a new salt
        let salt = get_salt(&pool, &employee_id).await?;
        assert!(salt.is_some());
        assert_eq!(SALT_LENGTH, salt.unwrap().len());
        assert_eq!(64, get_password(&pool, &employee_id).await?.len());

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../fixtures/truncate.sql",
        "../fixtures/address.sql",
        "../fixtures/employee.sql",
    ))]
    async fn test_verify_password_and_hash_password_on_updated_password(
        pool: PgPool,
    ) -> sqlx::Result<()> {
        // given is the employee_id and the password to verify on a user which has a proper
        // password
        let employee_id = 2;
        let given_password = String::from("catgirls123!");
        update_password(&pool, &employee_id, given_password.clone()).await?;

        // when verify_password is called
        let check = verify_password(&pool, &employee_id, given_password).await?;

        // then it should match since it is the first login and no salt was already existing
        assert!(check);

        Ok(())
    }
}
