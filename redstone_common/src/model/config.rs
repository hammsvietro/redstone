use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthData {
    pub cookies: Option<String> 
}

impl AuthData {
    pub fn new(cookies: String) -> Self {
        Self {
            cookies: Some(cookies)
        }
    }
}
