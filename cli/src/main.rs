mod keygen;
mod process;
mod zen_node;
mod decrypt;
use clap::Parser;
use keygen::KeygenCmd;
use process::StoreCmd;
use zen_node::ZenNodeCmd;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Parser, Debug, Clone)]
enum Commands {
    KeyGen(KeygenCmd),
    ProcessData(StoreCmd),
    ZenNode(ZenNodeCmd),
}

#[rocket::main]
async fn main() {
    let args = Args::parse();
    match args.cmd {
        Commands::ProcessData(store_cmd) => {
            if let Err(error) = store_cmd.execute().await {
                eprintln!("{}", error);
            }
        }
        Commands::KeyGen(key_gen) => {
            if let Err(error) = key_gen.execute().await {
                eprintln!("{}", error);
            }
        }
        Commands::ZenNode(zen_cmd) => {
            if let Err(error) = zen_cmd.execute().await {
                eprintln!("{}", error);
            }
        }
    }
}
