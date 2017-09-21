
mod clients_command;
mod run_command;
mod user_command;
mod error;

use clap;
use store;
use self::error::CliError;

pub fn dispatch_command(command: clap::ArgMatches, backend_store: Box<store::Store + Send + Sync>) {
    let result = match command.subcommand() {
        ("users", Some(command)) => user_command::handle_users_command(command, backend_store),
        ("clients", Some(command)) => {
            clients_command::handle_clients_command(command, backend_store)
        }
        ("run", Some(command)) => run_command::handle_run_command(command, backend_store),
        _ => Err(CliError::OtherError("unknown command")),
    };

    match result {
        Ok(_) => return,
        Err(e) => panic!("{}", e),
    }
}
