
use std::env;

fn main(){

    match env::var("CONFIG_DIR") {
        Ok(config_dir) =>println!("cargo:rustc-env=CONFIG_DIR={}",config_dir),
        Err(_) => {
            let mut home_dir = env::home_dir().expect("could not get home directory.");
            home_dir.push(".config");
            home_dir.push("openid-rs");
            println!("cargo:rustc-env=CONFIG_DIR={}",home_dir.to_string_lossy().to_lowercase());
        }
    }
}