use argon2::{self, Config};
use chrono::prelude::*;
use rand::Rng;
use std::{env, future};
use warp::{Filter};

use crate::store::Store;
use crate::types::accounts::{Account, AccountId, Session};

/*
@desc Register a new user.
@path POST /registration
@param account: Account struct with user information
@return: JSON response with "Account added" on success, or error
*/
pub async fn register(
    store: Store,
    account: Account
) -> Result<impl warp::Reply, warp::Rejection> {
    let hashed_password = hash_password(account.password.as_bytes());
    let account = Account {
        id: account.id,
        username: account.username,
        password: hashed_password,
        role: account.role
    };
    match store.add_account(account).await {
        Ok(_) => {
            Ok(warp::reply::json(&"Account added".to_string()))
        }
        Err(e) => Err(warp::reject::custom(e))
    }
}

/*
@desc Hash a password using Argon2 with random salt.
@param password: The password to hash
@return String containing the hashed password
*/
fn hash_password(password: &[u8]) -> String {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).unwrap()
}


/*
@desc Login user with username and password.
@path POST /login
@param login: Account struct with username and password
@return: JSON response with token on success, or error
*/
pub async fn login(
    store: Store,
    login: Account
) -> Result<impl warp::Reply, warp::Rejection> {
     match store.get_account(login.username).await {
         Ok(account) => match verify_password(
             &account.password,
             login.password.as_bytes()
         ) {
             Ok(verified) => {
                 if verified {
                     Ok(warp::reply::json(&issue_token(
                         account.id.expect("id not found"),
                         // account.role
                     )))
                 } else {
                     Err(warp::reject::custom(
                         handle_errors::Error::WrongPassword
                     ))
                 }
             }
             Err(e) => Err(warp::reject::custom(
                 handle_errors::Error::ArgonLibraryError(e),
             ))
         }
         Err(e) => Err(warp::reject::custom(e))
     }
}

/*
@desc Verify a password using Argon2.
@param hash: The hashed password to compare
@param password: The password to verify
@return true or argon2 error
*/
fn verify_password(
    hash: &str,
    password: &[u8],
) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password)
}

/*
@desc Create a PASETO token with account ID and expiration date.
@param account_id: The ID of the account
@return String containing the generated token
*/
fn issue_token(account_id: AccountId) -> String {
    let key = env::var("PASETO_KEY").unwrap();

    let current_date_time = Utc::now();
    let dt = current_date_time + chrono::Duration::days(1);

    paseto::tokens::PasetoBuilder::new()
        .set_encryption_key(&Vec::from(key.as_bytes()))
        .set_expiration(&dt)
        .set_not_before(&Utc::now())
        .set_claim("account_id", serde_json::json!(account_id))
        // .set_claim("role", serde_json::json!(role))
        .build()
        .expect("Failed to construct paseto token w/ builder!")
}

/*
@desc Authentication filter that verifies a PASETO token.
@return: Filter
 */
pub fn auth(
) -> impl Filter<Extract = (Session,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization").and_then(|token: String| {
        let token = match verify_token(token) {
            Ok(t) => t,
            Err(_) => return future::ready(Err(warp::reject::reject())),
        };

        future::ready(Ok(token))
    })
}

/*
@desc Funtion to verify a PASETO token
@param token
@return A session if token is validated, or error
 */
pub fn verify_token(
    token: String,
) -> Result<Session, handle_errors::Error> {
    let key = env::var("PASETO_KEY").unwrap();
    let token = paseto::tokens::validate_local_token(
        &token,
        None,
        key.as_bytes(),
        &paseto::tokens::TimeBackend::Chrono,
    )
        .map_err(|_| handle_errors::Error::CannotDecryptToken)?;

    serde_json::from_value::<Session>(token)
        .map_err(|_| handle_errors::Error::CannotDecryptToken)
}

// pub async fn is_admin(
//     token: String
// )

#[cfg(test)]
mod authentication_test {
    use super::{auth, env, issue_token, AccountId};

    #[tokio::test]
    async fn post_products_auth() {
        env::set_var("PASETO_KEY", "RANDOM WORDS WINTER MACINTOSH PC");
        let token = issue_token(AccountId(3));
        let filter = auth();
        let res = warp::test::request()
            .header("Authorization", token)
            .filter(&filter);
        assert_eq!(res.await.unwrap().account_id, AccountId(3));
    }
}