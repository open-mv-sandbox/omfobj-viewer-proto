mod commands;
mod io;

use clap::{Parser, Subcommand};
use commands::{add::AddCommandActor, create::CreateCommandActor};
use stewart_local::Dispatcher;
use stewart_runtime_native::NativeRuntime;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Parse command line args
    let args = CliArgs::parse();

    // Set up the runtime
    let runtime = NativeRuntime::new();
    let dispatcher = runtime.dispatcher();
    let start_addr = runtime.start_actor_manager();

    // Start the command actor
    let msg = match args.command {
        Command::Create(c) => CreateCommandActor::msg(c),
        Command::Add(c) => AddCommandActor::msg(dispatcher.clone(), start_addr, c),
    };
    // TODO: Wonky dispatcher syntax
    (dispatcher.as_ref() as &dyn Dispatcher).send(start_addr, msg);

    // Run until we're done
    runtime.block_execute();

    // TODO: Stewart doesn't currently bubble up errors for us to catch, and we need those for the
    // correct error code.
    /*if let Err(error) = result {
        event!(Level::ERROR, "failed:\n{:?}", error);
        std::process::exit(1);
    }*/
}

/// Pterodactil CLI toolkit for working with dacti packages.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Create(commands::create::CreateCommand),
    Add(commands::add::AddCommand),
}
