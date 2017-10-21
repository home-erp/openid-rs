#![feature(plugin,custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rustwt;
extern crate rusqlite;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate uuid;
extern crate time;
extern crate openssl;
extern crate url;
extern crate base64;
extern crate clap;


pub mod store;
pub mod command_dispatcher;
pub mod utils;
pub mod server;