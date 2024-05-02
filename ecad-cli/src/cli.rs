use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub sub: Subcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    #[command(subcommand, alias = "sl")]
    Schlib(CmdSchlib),
    #[command(subcommand, alias = "pl")]
    Pcblib(CmdPcblib),
}

#[derive(Debug, clap::Subcommand)]
pub enum CmdSchlib {
    List(LibListArgs),
}

#[derive(Debug, clap::Subcommand)]
pub enum CmdPcblib {
    List(LibListArgs),
}

#[derive(Debug, clap::Args)]
pub struct LibListArgs {
    /// Name of the file to open
    pub fname: PathBuf,
    /// Single item
    #[arg(short, long)]
    pub item: Vec<String>,
    /// List records for the items
    #[arg(short, long, default_value_t = false)]
    pub records: bool,

    #[arg(short, long, value_delimiter = ',')]
    pub filter: Vec<String>,
}
