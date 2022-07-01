/// Cookie Jar methods

use std::sync::Arc;
use reqwest::cookie::{Jar, CookieStore};
use super::get_api_base_url;
use super::super::config::get_auth_data;

pub fn get_jar() -> std::io::Result<Arc<Jar>> {
   let jar = Jar::default();
   let auth_data = get_auth_data()?;
   match auth_data {
       Some(auth_data) => match auth_data.cookies {
           Some(cookies) => jar.add_cookie_str(cookies.as_str(), &get_api_base_url()),
           _ => ()
       }
       _ => ()
   }
   jar.add_cookie_str("_redstone_server_web_user_remember_me=SFMyNTY.g2gDbQAAACAIJqcqmRSXHGnK76s4GjK-S5K7gdguWBFQ1Wt_M8BngG4GAAdghmuDAWIATxoA.leRSfliif3x5yBZOgSJgyeeVjur-zwnbg_4INbu5NCY; path=/; expires=Tue, 22 Nov 2022 18:05:12 GMT; max-age=5184000; HttpOnly; SameSite=Lax", &get_api_base_url());
   Ok(Arc::new(jar))
}

pub fn set_cookie(jar: Arc<Jar>, cookie: &str) -> () {
    let url = &get_api_base_url();
    jar.add_cookie_str(cookie, url);
    let cookies = jar.cookies(url);
    println!("\n\n\ncookies: {cookies:?}");
}
