#[derive(Debug, Clone, clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    Server,
    Client(ClientArgs),
}

#[derive(Debug, Clone, clap::Args)]
pub struct ClientArgs {}
