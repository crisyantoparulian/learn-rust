mod handlers;
mod routes;
mod models;

use routes::routes;
use models::blockchain::Blockchain;
use std::sync::Arc;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env file

    let blockchain = Arc::new(Blockchain::new());
    let api = routes(blockchain);

    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}