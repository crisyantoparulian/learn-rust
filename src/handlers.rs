use chrono::Duration;
use chrono::Utc;
use jsonwebtoken::encode;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use oauth2::PkceCodeVerifier;
use warp::Reply;
use std::env;
use std::sync::Arc;
use crate::models;
use models::blockchain::Blockchain;
use models::request::AddBlockRequest;
use models::google::{OAuthError, GoogleAuthResponse};
use models::jwt::CustomClaims;
use serde::Serialize;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, TokenResponse};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{ClientId, ClientSecret, RedirectUrl, TokenUrl, AuthUrl, Scope};
use hyper::Uri;
use std::sync::Mutex;
use std::collections::HashMap;



#[derive(Serialize)]
struct Response {
    message: String,
}

pub async fn get_chain(blockchain: Arc<Blockchain>, _claims: CustomClaims) -> Result<impl Reply, warp::Rejection> {
    let chain = blockchain.get_chain();
    Ok(warp::reply::json(&chain))
}

pub async fn add_block(req: AddBlockRequest, blockchain: Arc<Blockchain>) -> Result<impl Reply, warp::Rejection> {
    blockchain.add_block(req.data);
    Ok(warp::reply::json(&Response {
        message: "Block added".to_string(),
    }))
}

lazy_static::lazy_static! {
    static ref CODE_VERIFIERS: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub async fn google_auth_start() -> Result<impl Reply, warp::Rejection> {
    let client = create_google_oauth_client();

    // Generate a PKCE code verifier and challenge
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let state = CsrfToken::new_random();

    println!("Generated PKCE Verifier: {}", pkce_verifier.secret());
    println!("Generated State Token: {}", state.secret());

    // insert into memory for temp solution persistence database
    CODE_VERIFIERS.lock().unwrap().insert(state.secret().clone(), pkce_verifier.secret().clone());

    // Define the scopes your application needs
    let scopes = vec![
        Scope::new("openid".to_string()), // Required for OpenID Connect
        Scope::new("email".to_string()),  // Request access to the user's email
        Scope::new("profile".to_string()), // Request access to the user's profile information
    ];

    // Generate the authorization URL
    let (auth_url, csrf_token) = client
        .authorize_url(|| state)
        .set_pkce_challenge(pkce_challenge)
        .add_scopes(scopes)
        .url();

    println!("AUTH URL: {} , CSRF TOKEN: {}", auth_url.clone(), csrf_token.clone().secret());

    // Convert `Url` to `Uri`
    let auth_uri = Uri::from_maybe_shared(auth_url.to_string())
    .map_err(|_| warp::reject::custom(OAuthError))?;

    // Redirect the user to the authorization URL
    Ok(warp::redirect::temporary(auth_uri))
}

pub async fn google_auth_callback(query: GoogleAuthResponse) -> Result<impl Reply, warp::Rejection> {
    println!("Code: {}", query.code);
    println!("State: {}", query.state);

    let client = create_google_oauth_client();

    // Retrieve the PKCE verifier using state
    let verifier_opt = CODE_VERIFIERS.lock().unwrap().remove(&query.state);

    match &verifier_opt {
        Some(verifier_str) => println!("PKCE Verifier found: {}", verifier_str),
        None => {
            eprintln!("OAuth Error: Missing PKCE Verifier");
            return Err(warp::reject::custom(OAuthError));
        }
    }

    // Fix: Use `as_ref()` + `clone()`
    let pkce_verifier = PkceCodeVerifier::new(verifier_opt.as_ref().unwrap().clone());
    println!("KESINI 1: {}", pkce_verifier.secret());

    // Exchange the authorization code for a token
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            eprintln!("OAuth Token Exchange Error: {:?}", e);
            warp::reject::custom(OAuthError)
        })?;

    println!("MASUKK1 : {}",token_result.access_token().secret())
    ;

    // Use the token to fetch user information
    let user_info = reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
        .map_err(|_| warp::reject::custom(OAuthError))?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| warp::reject::custom(OAuthError))?;

    // Extract user ID & email
    let user_id = user_info["sub"].as_str().unwrap_or("");
    let email = user_info["email"].as_str().unwrap_or("");

    let cust_jwt = generate_jwt(user_id, email);

    // Return the custom JWT to the frontend
    Ok(warp::reply::json(&serde_json::json!({
        "access_token": cust_jwt,
        "token_type": "Bearer",
        "expires_in": 3600
    })))


    // Return the user information as a response
    // Ok(warp::reply::json(&user_info))

    // Ok(warp::reply::json(&serde_json::json!({
    //     "access_token": token_result.access_token().secret(),
    //     "expires_in": token_result.expires_in().unwrap_or_default(),
    //     "token_type": "Bearer"
    // })))
}

fn create_google_oauth_client() -> BasicClient {
    let env_client_id = env::var("GOOGLE_CLIENT_ID").expect("google client id not found in env");
    let env_client_secret = env::var("GOOGLE_CLIENT_SECRET").expect("google client secret not found in env");
    let env_client_auth_url = env::var("GOOGLE_AUTH_URL").expect("google client auth url not found in env");
    let env_client_token_url = env::var("GOOGLE_TOKEN_URL").expect("google client token url not found in env");
    let env_client_redirect_url = env::var("AUTH_CALLBACK_REDIRECT_URL").expect("google client redirect url not found in env");

    let client_id = ClientId::new(env_client_id.to_string());
    let client_secret = ClientSecret::new(env_client_secret.to_string());
    let auth_url = AuthUrl::new(env_client_auth_url.to_string()).unwrap();

    let token_url = TokenUrl::new(env_client_token_url.to_string()).unwrap();
    let redirect_url = RedirectUrl::new(env_client_redirect_url.to_string()).unwrap();

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url)
}

fn generate_jwt(user_id: &str, email: &str) -> String {
    let jwt_secret = env::var("JWT_SECRET").expect("jwt secret not found in env");
    let expiration = Utc::now() + Duration::hours(1);
    let claims = CustomClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: expiration.timestamp() as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref())).
        expect("JWT encoding failed")
}