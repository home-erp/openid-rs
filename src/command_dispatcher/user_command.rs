use clap;
use store::{self, Store};
use command_dispatcher::error::CliError;
use std::{self, fs};
use std::io::prelude::*;
use uuid;
use openssl;
use base64;

pub fn handle_users_command(command: &clap::ArgMatches, store: Box<Store>) -> Result<(), CliError> {
    match command.subcommand() {
        ("add", Some(sub_command)) => handle_add_user_command(sub_command, store),
        ("list", Some(_)) => handle_list_command(store),
        ("delete", Some(sub_command)) => handle_delete_user_command(sub_command, store),
        ("change-email", Some(sub_command)) => handle_change_email_command(sub_command, store),
        ("join-group", Some(sub_command)) => handle_join_group_command(sub_command, store),
        ("leave-group", Some(sub_command)) => handle_leave_group_command(sub_command, store),
        _ => {
            eprintln!("require at least one subcommand!");
            std::process::exit(1);
        }
    }
}

fn handle_leave_group_command(args: &clap::ArgMatches, store: Box<Store>) -> Result<(), CliError> {
    let email = args.value_of("REFERENCE").unwrap();
    let group = args.value_of("GROUP").unwrap();
    store.remove_group(email, group)?;
    Ok(())
}

fn handle_join_group_command(args: &clap::ArgMatches, store: Box<Store>) -> Result<(), CliError> {
    let email = args.value_of("REFERENCE").unwrap();
    let group = args.value_of("GROUP").unwrap();
    store.add_group(email, group)?;
    Ok(())
}

//TODO implement
fn handle_change_email_command(_: &clap::ArgMatches, _: Box<Store>) -> Result<(), CliError> {
    Err(CliError::OtherError("not implemented yet"))
}

fn handle_list_command(store: Box<Store>) -> Result<(), CliError> {
    let users = store.get_users()?;
    for user in users {
        println!("Email: {}", user.email);
        if !user.groups.is_empty() {
            println!("Groups: {}", user.groups.join(","));
        }
    }
    Ok(())
}


fn handle_delete_user_command(args: &clap::ArgMatches, store: Box<Store>) -> Result<(), CliError> {
    let email = args.value_of("REFERENCE").unwrap();
    store.delete_user(email)?;
    Ok(())
}


fn handle_add_user_command(args: &clap::ArgMatches, store: Box<Store>) -> Result<(), CliError> {
    let email = args.value_of("EMAIL").unwrap();
    let groups = match args.values_of("group") {
        Some(gr) => gr.map(|item| item.to_string()).collect(),
        None => Vec::new(),
    };

    let mut pwd = match args.value_of("password") {
        Some(pwd) => pwd.to_string(),
        None => {
            if let Some(pwd_file) = args.value_of("password-file") {
                read_pwd_from_file(pwd_file)?
            } else {
                ask_for_pwd()?
            }
        }
    };


    let mut salt_file_path = std::env::home_dir().ok_or(CliError::OtherError(
        "could not determine home directory",
    ))?;

    salt_file_path.push(".config");
    salt_file_path.push("openid-rs");
    salt_file_path.push("private");
    salt_file_path.push("salt.txt");
    std::fs::create_dir_all(salt_file_path.parent().unwrap())?;


    let mut salt = String::new();

    if salt_file_path.exists() {
        let mut salt_file = fs::File::open(salt_file_path)?;
        salt_file.read_to_string(&mut salt)?;
    } else {
        salt = uuid::Uuid::new_v4().simple().to_string();
        let mut salt_file = fs::File::create(salt_file_path)?;
        write!(salt_file, "{}", salt)?;
    }

    pwd.push_str(&salt[..]);

    let hashed_pwd_bytes = openssl::sha::sha256(pwd.as_bytes());

    let hashed_pwd = base64::encode(&hashed_pwd_bytes);

    let user = store::User {
        email: String::from(email),
        password: Some(hashed_pwd),
        groups: groups,
    };
    store.save_user(&user)?;
    Ok(())
}

fn read_pwd_from_file(file: &str) -> Result<String, CliError> {
    let mut pwd_file = fs::File::open(file)?;
    let mut content = String::new();
    pwd_file.read_to_string(&mut content)?;
    Ok(content)
}

fn ask_for_pwd() -> Result<String, CliError> {
    print!("enter user password\n>");
    let stdin = std::io::stdin();
    let line = stdin.lock().lines().next().expect("no password entered.")?;
    Ok(String::from(line.trim()))
}
