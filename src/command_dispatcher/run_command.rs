use clap;
use std::{self, fs};
use utils::get_path;
use store::Store;
use command_dispatcher::error::CliError;
use openssl::ec::{EcGroup, NAMED_CURVE, EcKey};
use openssl::nid::X9_62_PRIME256V1;
use openssl::pkey::PKey;
use std::sync::RwLock;
use std::collections::HashMap;
use server;
use std::io::prelude::*;
use uuid;


pub fn handle_run_command(
    command: &clap::ArgMatches,
    store: Box<Store + Send + Sync>,
) -> Result<(), CliError> {
    let home_dir = std::env::home_dir().ok_or(CliError::OtherError(
        "could not determine home directory",
    ))?;

    let config_dir = get_path(&home_dir, &[".config", "openid-rs"]);

    let listen = command.value_of("address").unwrap_or("0.0.0.0");

    let issuer = command.value_of("issuer").map(|i| String::from(i));

    let port = command
        .value_of("port")
        .map(|item| item.parse::<u16>())
        .unwrap_or(Ok(8080))?;


    let private_dir = get_path(&config_dir, &["private"]);
    let sign_key = get_path(&private_dir, &["sign-key.pem"]);
    let verification_key_path = get_path(&config_dir, &["verification-key.pem"]);
    let salt_file_path = get_path(&private_dir, &["salt.txt"]);
    fs::create_dir_all(sign_key.parent().unwrap())?;

    let key_pair = if !sign_key.exists() {
        let mut group = EcGroup::from_curve_name(X9_62_PRIME256V1)?;
        group.set_asn1_flag(NAMED_CURVE);
        let key = EcKey::generate(&group)?;
        let key = PKey::from_ec_key(key)?;

        let pem = key.private_key_to_pem()?;
        let public_pem = key.public_key_to_pem()?;
        let _ = fs::remove_file(verification_key_path.clone());
        let mut sign_key_file = fs::File::create(sign_key)?;
        let mut verification_key_file = fs::File::create(verification_key_path)?;
        sign_key_file.write_all(&pem)?;
        verification_key_file.write_all(&public_pem)?;
        key
    } else {
        let mut private_key_file = fs::File::open(sign_key)?;
        let mut content = Vec::new();
        private_key_file.read_to_end(&mut content)?;
        PKey::private_key_from_pem(&content)?
    };

    let mut salt = String::new();

    if salt_file_path.exists() {
        let mut salt_file = fs::File::open(salt_file_path)?;
        salt_file.read_to_string(&mut salt)?;
    } else {
        salt = uuid::Uuid::new_v4().simple().to_string();
        let mut salt_file = fs::File::create(salt_file_path)?;
        write!(salt_file, "{}", salt)?;
    }

    let app_config = server::Config {
        issuer: issuer,
        config_dir_path: String::from(config_dir.to_str().ok_or(CliError::OtherError(
            "could not convert path to string",
        ))?),
        store: store,
        sessions: RwLock::new(HashMap::new()),
        token_duration: 7 * 24 * 60 * 60,
        codes: RwLock::new(HashMap::new()),
        salt: salt,
        key_pair: key_pair,
    };
    server::run(app_config, listen, port);
    Ok(())
}
