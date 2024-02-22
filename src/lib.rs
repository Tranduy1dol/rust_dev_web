#![warn(clippy::all)]

use warp::{http::Method, Filter};
use tokio::sync::{oneshot, oneshot::Sender};
use tracing_subscriber::fmt::format::FmtSpan;

mod routes;
mod store;
pub mod types;
pub mod config;
pub use handle_errors;

pub struct OneshotHandler {
    pub sender: Sender<i32>
}

/*
@desc Function to build the main API routes
 */
async fn build_routes(store: store::Store) -> impl Filter<Extract = impl warp::Reply> + Clone {
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[
            Method::PUT,
            Method::DELETE,
            Method::GET,
            Method::POST,
        ]);
    //POST method
    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::login);

    let add_product = warp::post()
        .and(warp::path("products"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::products::add_product);

    //GET method
    let get_product = warp::get()
        .and(warp::path("products"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::products::get_products);

    //PUT method
    let update_product = warp::put()
        .and(warp::path("products"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::products::update_product);

    //DELETE method
    let delete_product = warp::delete()
        .and(warp::path("products"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::products::delete_product);


    registration
        .or(login)
        .or(get_product)
        .or(add_product)
        .or(update_product)
        .or(delete_product)
        .with(cors)
        .with(warp::trace::request())
        .recover(handle_errors::return_error)
}


/*
@desc Function to set up database store based on configuation
 */
pub async fn setup_store(config: &config::Config) -> Result<store::Store, handle_errors::Error> {
    let store = store::Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_user, config.db_password, config.db_host, config.db_port, config.db_name))
        .await
        .map_err(|e| handle_errors::Error::DatabaseQueryError(e))?;

    let _ = sqlx::migrate!()
        .run(&store.clone().connection)
        .await
        .map_err(|e| handle_errors::Error::MigrationError(e));

    let log_filter = format!(
        "handle_errors={}, restful-api={}, warp={}",
        config.log_level, config.log_level, config.log_level
    );

    tracing_subscriber::fmt()
        //Use the filter we built above to determine which traces to record
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    Ok(store)
}

/*
@desc Entry point for running the API server.
 */
pub async fn run(config: config::Config, store: store::Store) {
    let routes = build_routes(store).await;
    warp::serve(routes).run(([127, 0, 0, 1], config.port)).await;
}

/*
@desc Function to create a one-shot API server.
 */
pub async fn oneshot(store: store::Store) -> OneshotHandler {
    let routes = build_routes(store).await;
    let (tx, rx) = oneshot::channel::<i32>();

    let socket: std::net::SocketAddr = "127.0.0.1:3030"
        .to_string()
        .parse()
        .expect("Not a valid address");

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(socket, async {
        rx.await.ok();
    });

    tokio::task::spawn(server);
    OneshotHandler {
        sender: tx
    }
}