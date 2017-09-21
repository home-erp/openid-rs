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
extern crate clap;
extern crate base64;


mod store;
mod command_dispatcher;
mod utils;
mod server;

use clap::{Arg, App, SubCommand, AppSettings};


use std::fs;
use store::sqlite_store::SqliteStore;


const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");



fn main() {

    let matches = App::new("openid-rs")
        .version(VERSION)
        .author(AUTHORS)
        .setting(AppSettings::SubcommandRequired)
        .about("an openid connect provider")
        .subcommand(
            SubCommand::with_name("run")
                .about("run the server.")
                .arg(
                    Arg::with_name("address")
                        .short("a")
                        .long("address")
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .help(
                            "Sets the address this instance is listening to. Defaults to 0.0.0.0",
                        ),
                )
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .value_name("PORT")
                        .takes_value(true)
                        .help(
                            "Sets the port this instance is listening on. Defaults to 8080",
                        ),
                )
                .arg(
                    Arg::with_name("issuer")
                        .short("i")
                        .long("issuer")
                        .value_name("ISSUER")
                        .help(
                            "Sets the token issuer. \
                    If not set, the issuer is set to the incoming requests host name.",
                        )
                        .takes_value(true),
                ),
        )
        .subcommand(users_subcommand())
        .subcommand(clients_subcommand())
        .get_matches();

    let home_dir = std::env::home_dir().unwrap();
    let db_path = utils::get_path(&home_dir, &[".local", "share", "openid-rs", "db.sqlite3"]);
    fs::create_dir_all(db_path.parent().unwrap()).expect("could not create database directory.");
    let db_path_str = db_path.to_str().expect(
        "could not convert db-path to string",
    );

    let store = match SqliteStore::new(db_path_str) {
        Ok(store) => Box::new(store),
        Err(e) => panic!("error while creating backend store: {}", e),
    };

    command_dispatcher::dispatch_command(matches, store);

}



fn clients_subcommand() -> clap::App<'static, 'static> {
    SubCommand::with_name("clients")
        .setting(AppSettings::SubcommandRequired)
        .about("control clients")
        .subcommand(
            SubCommand::with_name("add")
                .arg(Arg::with_name("NAME").required(true).help(
                    "Name of the new client",
                ))
                .arg(
                    Arg::with_name("redirect-url")
                        .short("r")
                        .long("redirect-url")
                        .multiple(true)
                        .value_name("REDIRECT_URL")
                        .help("add a redirect url to this client"),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("show clients"))
        .subcommand(SubCommand::with_name("delete").arg(
            Arg::with_name("REFERENCE").help(
                "A reference to a client. Either the ID of the name of the client.",
            ),
        ))
        .subcommand(
            SubCommand::with_name("add-redirect-url")
                .about("change the name of a client")
                .arg(Arg::with_name("REFERENCE").required(true).help(
                    "A reference to a client. Either the ID of the name of the client.",
                ))
                .arg(Arg::with_name("URL").required(true).help(
                    "the redirect url to add",
                )),
        )
        .subcommand(
            SubCommand::with_name("remove-redirect-url")
                .about("change the name of a client")
                .arg(Arg::with_name("REFERENCE").required(true).help(
                    "A reference to a client. Either the ID of the name of the client.",
                ))
                .arg(Arg::with_name("URL").required(true).help(
                    "the redirect url to remove",
                )),
        )
}



fn users_subcommand() -> clap::App<'static, 'static> {
    SubCommand::with_name("users")
        .about("control users")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("add")
                .arg(
                    Arg::with_name("EMAIL")
                        .help("the email address of the user")
                        .required(true),
                )
                .arg(
                    Arg::with_name("password")
                        .short("p")
                        .long("password")
                        .value_name("PASSWORD")
                        .takes_value(true)
                        .help(
                            "The new users password. \
                    If neither this nor the --password-file options is set, \
                    the password will be read from stdin.",
                        ),
                )
                .arg(
                    Arg::with_name("password-file")
                        .short("f")
                        .long("password-file")
                        .value_name("FILE")
                        .takes_value(true)
                        .help(
                            "The new users password, read from a file. \
                            If neither this nor the --password options is set, \
                            the password will be read from stdin.",
                        ),
                )
                .arg(
                    Arg::with_name("group")
                        .short("g")
                        .long("group")
                        .takes_value(true)
                        .value_name("GROUP")
                        .multiple(true)
                        .help(
                            "Add a group for this user.
                    If it does not exist, it will be created.",
                        ),
                ),
        )
        .subcommand(SubCommand::with_name("list").help("List available users."))
        .subcommand(SubCommand::with_name("delete").arg(
            Arg::with_name("REFERENCE").help("Id or email of user"),
        ))
        .subcommand(
            SubCommand::with_name("change-email")
                .arg(Arg::with_name("REFERENCE").required(true).help(
                    "Id or email of user",
                ))
                .arg(Arg::with_name("NEW_EMAIL").required(true).help(
                    "The new email Address",
                )),
        )
        .subcommand(
            SubCommand::with_name("join-group")
                .arg(Arg::with_name("REFERENCE").required(true).help(
                    "Id or email of user",
                ))
                .arg(Arg::with_name("GROUP").required(true).help(
                    "The group the user should belong to.
                    If it does not exist, it will becreated.",
                )),
        )
        .subcommand(
            SubCommand::with_name("leave-group")
                .arg(Arg::with_name("REFERENCE").required(true).help(
                    "Id or email of user",
                ))
                .arg(Arg::with_name("GROUP").required(true).help(
                    "The group the user must leave.",
                )),
        )
}
