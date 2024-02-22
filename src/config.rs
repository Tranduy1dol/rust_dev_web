use clap::Parser;
use dotenv;
use std::env;


/*
@desc Configuration struct, using Args for more control
 */
#[derive(Parser, Debug, PartialEq)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    #[clap(short, long, default_value = "warn")]
    pub log_level: String, ///Log level, default: "warn"

    #[clap(short, long, default_value = "3030")]
    pub port: u16, //Server port, default: 3030

    #[clap(long, default_value = "postgres")]
    pub db_user: String, //Database username, default: "postgres"

    #[clap(long, default_value = "password")]
    pub db_password: String, //Database password, default: "password"

    #[clap(long, default_value = "localhost")]
    pub db_host: String, //Database host, default: "localhost"

    #[clap(long, default_value = "5432")]
    pub db_port: u16, //Database port, default: 5432

    #[clap(long, default_value = "data")]
    pub db_name: String //Database name, default: "data"
}

impl Config {
    pub fn new() -> Result<Config, handle_errors::Error> {
        //Load environment variables from .env file
        dotenv::dotenv().ok();

        //Parse command-line arguments
        let config = Config::parse();

        // if let Err(_) = env::var("BAD_WORDS_API_KEY") {
        //     panic!("Bad word api key not set");
        // }

        if let Err(_) = env::var("PASETO_KEY") {
            panic!("Paseto key not set");
        }

        //Handle evironment variables
        let port = std::env::var("PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(config.port))
            .map_err(|e| handle_errors::Error::ParseError(e))?;

        let db_user = env::var("DB_USER").unwrap_or_else(|_| config.db_user.to_owned());
        let db_password = env::var("DB_PASSWORD").unwrap();
        let db_host = env::var("DB_HOST").unwrap_or_else(|_| config.db_host.to_owned());
        let db_port = env::var("DB_PORT").unwrap_or_else(|_| config.db_port.to_string());
        let db_name = env::var("DB_NAME").unwrap_or_else(|_| config.db_name.to_owned());

        Ok(Config {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port: db_port.parse::<u16>().map_err(|e| handle_errors::Error::ParseError(e))?,
            db_name
        })
    }
}

#[cfg(test)]
mod config_test {
    use super::*;

    fn set_env() {
        env::set_var("BAD_WORDS_API_KEY", "API_KEY");
        env::set_var("PASETO_KEY", "RANDOM WORDS WINTER MACINTOSH PC");
        env::set_var("DB_USER", "user");
        env::set_var("DB_PASSWORD", "pass");
        env::set_var("DB_HOST", "localhost");
        env::set_var("DB_PORT", "5432");
        env::set_var("DB_NAME", "data");
    }


    #[test]
    fn unset_and_set_api_kei() {
        let result = std::panic::catch_unwind(|| Config::new());
        assert!(result.is_err());

        set_env();
        let expected = Config {
            log_level: "warn".to_string(),
            port: 8080,
            db_user: "user".to_string(),
            db_password: "pass".to_string(),
            db_host: "localhost".to_string(),
            db_port: 5432,
            db_name: "data".to_string(),
        };
        let config = Config::new().unwrap();
        assert_eq!(config, expected);
    }
}