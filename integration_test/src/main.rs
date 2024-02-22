use futures_util::FutureExt;
use restful_api::{config, handle_errors, oneshot, setup_store};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use std::io::{self, Write};
use inline_colorization::*;


#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    username: String,
    password: String,
    role: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Token(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Product {
    name: String,
    price: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProductSell {
    id: i32,
    name: String,
    price: i32,
}

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();
    let config = config::Config::new().expect("Config can't be set");

    let s = Command::new("sqlx")
        .arg("database")
        .arg("drop")
        .arg("--database-url")
        .arg(format!("postgres://{}:{}@{}:{}/{}",
            config.db_user, config.db_password,config.db_host, config.db_port, config.db_name))
        .arg("-y")
        .output()
        .expect("sqlx command failed to start");

    io::stdout().write_all(&s.stderr).unwrap();

    //execute DB commands to drop and create a new test database
    let s = Command::new("sqlx")
        .arg("database")
        .arg("create")
        .arg("--database-url")
        .arg(format!("postgres://{}:{}@{}:{}/{}",
            config.db_user, config.db_password, config.db_host, config.db_port, config.db_name))
        .output()
        .expect("sqlx command failed to start");

    io::stdout().write_all(&s.stderr).unwrap();

    //Set up a new store instance with a db connection pool
    let store = setup_store(&config).await?;

    //start the server and listen for a sender signal to shut it down
    let handler = oneshot(store).await;

    //Create a user throughout the test
    let u = User {
        username: "username".to_string(),
        password: "password".to_string(),
        role: "user".to_string()
    };

    let token;

    print!("Running register_new_user...");
    let result = std::panic::AssertUnwindSafe(register_new_user(&u)).catch_unwind().await;
    match result {
        Ok(_) => println!("{color_green} Test pass ✓{color_reset}"),
        Err(_) => {
            let _ = handler.sender.send(1);
            std::process::exit(1);
        }
    }
    print!("Running login ...");
    match std::panic::AssertUnwindSafe(login(u)).catch_unwind().await {
        Ok(t) => {
            token = t;
            println!("{color_green} Test pass ✓{color_reset}");
        },
        Err(_) => {
            let _ = handler.sender.send(1);
            std::process::exit(1);
        }
    }

    print!("Running add_product ...");
    match std::panic::AssertUnwindSafe(add_product(token)).catch_unwind().await {
        Ok(_) => println!("{color_green} Test pass ✓{color_reset}"),
        Err(_) => {
            let _ = handler.sender.send(1);
            std::process::exit(1);
        }
    }
    let _ = handler.sender.send(1);
    Ok(())
}

async fn register_new_user(user: &User) {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3030/registration")
        .json(&user)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(res, "Account added".to_string());
}

async fn login(user: User) -> Token{
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3030/login")
        .json(&user)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    res.json::<Token>().await.unwrap()
}

async fn add_product(token: Token) {
    let p = Product {
        name: "sample".to_string(),
        price: 10
    };
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:3030/products")
        .header("Authorization", token.0)
        .json(&p)
        .send()
        .await
        .unwrap()
        .json::<ProductSell>()
        .await
        .unwrap();
    assert_eq!(res.id, 1);
    assert_eq!(res.name, p.name);
}