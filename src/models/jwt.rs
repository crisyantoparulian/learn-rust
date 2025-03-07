use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomClaims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
}

