use std::collections::HashMap;

use tracing::{event, instrument, Level};
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::accounts::Session;
use crate::types::pagination::{extract_pagination, Pagination};
use crate::types::products::{NewProducts, Products};

/*
@desc get a limit number of products
@path GET /products
 */
#[instrument]
pub async fn get_products(
    params: HashMap<String, String>,
    store: Store
) -> Result<impl warp::Reply, warp::Rejection> {
    event!(target: "restful-api", Level::INFO, "querying products");
    let mut pagination = Pagination::default();
    if !params.is_empty() {
        event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
    }
    
    match store.get_product(pagination.limit, pagination.offset).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e))
    }
}

/*
@desc update product information
@path PUT /products
 */
pub async fn update_product(
    id: i32,
    session: Session,
    store: Store,
    products: Products
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    if store.is_product_owner(id, &account_id).await? {
        match store.update_product(products, id, account_id.0).await {
            Ok(res) => Ok(warp::reply::json(&res)),
            Err(e) => Err(warp::reject::custom(e))
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}

/*
@desc Delete a product
@path DELETE /products
 */
pub async fn delete_product(
    id: i32,
    session: Session,
    store: Store
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    if store.is_product_owner(id, &account_id).await? {
        match store.delete_product(id, account_id).await {
            Ok(_) => Ok(warp::reply::with_status(
                format!("Question {} deleted", id),
                StatusCode::OK
            )),
            Err(e) => Err(warp::reject::custom(e))
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}

/*
@desc Add a new product
@path POST /products
 */
pub async fn add_product(
    session: Session,
    store: Store,
    new_products: NewProducts
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    let product = NewProducts {
        name: new_products.name,
        price: new_products.price,
    };
    match store.add_product(product, account_id).await {
        Ok(product) => Ok(warp::reply::json(&product)),
        Err(e) => Err(warp::reject::custom(e))
    }
}