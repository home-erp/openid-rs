
use store::Store;
use std::sync::RwLock;
use std::collections::HashMap;
use rustwt;
use rocket::{self, config};
use openssl;

pub mod routes;
mod authentication_request;

pub struct Config {
    pub issuer: Option<String>,
    pub config_dir_path: String,
    pub store: Box<Store + Send + Sync>,
    pub sessions: RwLock<HashMap<String, String>>,
    pub codes: RwLock<HashMap<String, rustwt::id_token::IDToken>>,
    pub token_duration: u64,
    pub salt: String,
    pub key_pair: openssl::pkey::PKey,
}

pub fn run(con: Config, listen: &str, port: u16) {

    let rocket_config = config::Config::build(config::Environment::Production)
        .address(listen)
        .port(port)
        .finalize()
        .expect("could not create rocket config");

    rocket::custom(rocket_config, false)
        .manage(con)
        .mount(
            "/",
            routes![routes::login, routes::authorize, routes::public_key],
        )
        .launch();
}
