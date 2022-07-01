use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AuthData {
    pub cookies: Option<String> 
}
