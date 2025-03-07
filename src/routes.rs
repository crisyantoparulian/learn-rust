use jsonwebtoken::{decode, DecodingKey, Validation};
use warp::Filter;
use crate::handlers::{get_chain, add_block, google_auth_start, google_auth_callback};
use crate::models;
use crate::models::google::OAuthError;
use models::blockchain::Blockchain;
use models::google::GoogleAuthResponse;
use models::jwt::CustomClaims;
use std::env;
use std::sync::Arc;

pub fn routes(blockchain: Arc<Blockchain>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let blockchain_clone = Arc::clone(&blockchain);
    let get_chain = warp::path!("chain")
        .and(warp::get())
        .and(warp::any().map(move || Arc::clone(&blockchain_clone)))
        .and(auth_filter())
        .and_then(get_chain);

    let blockchain_clone = Arc::clone(&blockchain);
    let add_block = warp::path!("add")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || Arc::clone(&blockchain_clone)))
        .and_then(add_block);

    let google_auth_start = warp::path!("auth" / "google")
        .and(warp::get())
        .and_then(google_auth_start);

    let google_auth_callback = warp::path!("auth" / "google" / "callback")
        .and(warp::get())
        .and(warp::query::<GoogleAuthResponse>())
        .and_then(google_auth_callback); 
    
    get_chain.or(add_block).or(google_auth_start).or(google_auth_callback)
}

fn auth_filter() -> impl Filter<Extract = (CustomClaims,), Error = warp::Rejection> + Clone {
    warp::header::<String>("authorization")
        .map(|header: String| header.trim_start_matches("Bearer ").to_string()) // Extract token
        .and_then(|token: String| async move {
            validate_jwt(token.as_ref()).map_err(|_| warp::reject::custom(OAuthError))
        })
}

fn validate_jwt(token: &str) -> Result<CustomClaims, warp::Rejection> {
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    let jwt_secret = env::var("JWT_SECRET").expect("jwt secret not found in env");

    decode::<CustomClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation
    )
    .map(|data| data.claims)
    .map_err(|_| warp::reject::custom(OAuthError))
}