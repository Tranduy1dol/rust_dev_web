use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow},
    Row,
};

use handle_errors::Error;

use crate::types::{
    accounts::{Account, AccountId},
};
use crate::types::products::{NewProducts, ProductId, Products};

#[derive(Debug, Clone)]
pub struct Store {
    pub(crate) connection: PgPool,
}

impl Store {

    ///Connect to PostgresQL with database url
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        tracing::warn!("{}", db_url);
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;
        Ok(Store{
            connection: db_pool
        })
    }

    ///Insert to database new account
    pub async fn add_account(
        self,
        account: Account
    ) -> Result<bool, Error> {
        match sqlx::query(
            "INSERT INTO accounts (username, password, role) VALUES ($1, $2, $3)"
        )
            .bind(account.username)
            .bind(account.password)
            .bind(account.role)
            .execute(&self.connection)
            .await {
            Ok(_) => Ok(true),
            Err(error) => {
                tracing::event!(
                    tracing::Level::ERROR,
                    code = error
                    .as_database_error()
                    .unwrap()
                    .code()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap(),
                    db_message =
                        error.as_database_error().unwrap().message(),
                    constraint = error
                        .as_database_error()
                        .unwrap()
                        .constraint()
                        .unwrap()
                );
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    ///Get accounts from database
    pub async fn get_account(
        self,
        username: String
    ) -> Result<Account, Error> {
        match sqlx::query("SELECT * FROM accounts WHERE username = $1")
            .bind(username)
            .map(|row: PgRow| Account {
                id: Some(AccountId(row.get("id"))),
                username: row.get("username"),
                password: row.get("password"),
                role: row.get("role")
            })
            .fetch_one(&self.connection)
            .await {
            Ok(account) => Ok(account),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    ///Get a limit number of products from database
    pub async fn get_product(
        self,
        limit: Option<i32>,
        offset: i32
    ) -> Result<Vec<Products>, Error> {
        match sqlx::query("SELECT * FROM products LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| Products {
                id: ProductId(row.get("id")),
                name: row.get("name"),
                price: row.get("price"),
            })
            .fetch_all(&self.connection)
            .await {
            Ok(productions) => Ok(productions),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError(e))
            }
        }
    }

    ///Add a new product to database
    pub async fn add_product(
        self,
        new_productions: NewProducts,
        account_id: AccountId
    ) -> Result<Products, Error> {
        match sqlx::query("INSERT INTO products (name, price, seller_id) VALUES ($1, $2, $3) RETURNING id, name, price, seller_id")
            .bind(new_productions.name)
            .bind(new_productions.price)
            .bind(account_id.0)
            .map(|row: PgRow| Products {
                id: ProductId(row.get("id")),
                name: row.get("name"),
                price: row.get("price"),
            })
            .fetch_one(&self.connection)
            .await {
            Ok(production) => Ok(production),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }

    ///Update a product information
    pub async fn update_product(
        self,
        product: Products,
        id: i32,
        seller_id: i32
    ) -> Result<Products, Error> {
        match sqlx::query("UPDATE products SET name = $1, price = $2\
        WHERE id = $3 AND seller_id = $4 RETURNING id, name, price")
            .bind(product.name)
            .bind(product.price)
            .bind(id)
            .bind(seller_id)
            .map(|row: PgRow| Products {
                id: ProductId(row.get("id")),
                name: row.get("name"),
                price: row.get("price"),
            })
            .fetch_one(&self.connection)
            .await {
            Ok(product) => Ok(product),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }


    ///Delete a product in database
    pub async fn delete_product(
        self,
        id: i32,
        seller_id: AccountId
    ) -> Result<bool, Error> {
        match sqlx::query("DELETE FROM products WHERE id = $1 AND seller_id = $2")
            .bind(id)
            .bind(seller_id.0)
            .execute(&self.connection)
            .await {
            Ok(_) => Ok(true),
            Err(error) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", error);
                Err(Error::DatabaseQueryError(error))
            }
        }
    }


    ///Verify that a user is product owner or not, so they can change product information in database
    pub async fn is_product_owner(
        &self,
        product_id: i32,
        account_id: &AccountId
    ) -> Result<bool, Error> {
        match sqlx::query("SELECT * FROM products WHERE id = $1 AND seller_id = $2")
            .bind(product_id)
            .bind(account_id.0)
            .fetch_optional(&self.connection)
            .await
        {
            Ok(product) => Ok(product.is_some()),
            Err(e) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", e);
                Err(Error::DatabaseQueryError(e))
            }
        }
    }
}