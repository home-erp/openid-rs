use clap;
use store::{Store, Client};
use command_dispatcher::error::CliError;

pub fn handle_clients_command(
    command: &clap::ArgMatches,
    store: Box<Store>,
) -> Result<(), CliError> {

    match command.subcommand() {
        ("add", Some(args)) => handle_add_client_command(args, store),
        ("delete", Some(args)) => handle_delete_client_command(args, store),
        ("add-redirect-url", Some(args)) => handle_add_redirect_command(args, store),
        ("remove-redirect-url", Some(args)) => handle_remove_redirect_command(args, store),
        ("list", Some(_)) => handle_list_clients_command(store),
        _ => panic!("unknown command"),
    }
}

fn handle_list_clients_command(store: Box<Store>) -> Result<(), CliError> {
    let clients = store.get_clients()?;

    for client in clients {
        println!("{}", client.name);
        println!("{}", client.redirect_urls.join(" "));
    }
    Ok(())
}


fn handle_add_client_command(args: &clap::ArgMatches, store: Box<Store>) -> Result<(), CliError> {
    let client_name = args.value_of("NAME").unwrap(); //safe unwrap
    let urls = match args.values_of("redirect-url") {
        Some(urls) => urls.map(|item| item.to_string()).collect(),
        None => Vec::new(),
    };
    let client = Client {
        name: String::from(client_name),
        redirect_urls: urls,
    };
    store.save_client(&client)?;
    Ok(())
}


fn handle_delete_client_command(
    args: &clap::ArgMatches,
    store: Box<Store>,
) -> Result<(), CliError> {
    let name = args.value_of("REFERENCE").unwrap();
    store.delete_client(name)?;
    Ok(())
}

fn handle_add_redirect_command(args: &clap::ArgMatches, store: Box<Store>) -> Result<(), CliError> {
    let name = args.value_of("REFERENCE").unwrap();
    let url = args.value_of("URL").unwrap();
    store.add_redirect_url(name, url)?;
    Ok(())
}

fn handle_remove_redirect_command(
    args: &clap::ArgMatches,
    store: Box<Store>,
) -> Result<(), CliError> {
    let name = args.value_of("REFERENCE").unwrap();
    let url = args.value_of("URL").unwrap();
    store.remove_redirect_url(name, url)?;
    Ok(())
}
