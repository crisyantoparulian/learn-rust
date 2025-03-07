
#[derive(serde::Deserialize)]
pub struct GoogleAuthResponse {
    pub code: String,
    pub state: String,
}

#[derive(Debug)]
pub struct OAuthError;

impl warp::reject::Reject for OAuthError {}