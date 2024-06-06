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
    #[command(subcommand, alias = "s")]
    Schdoc(CmdSchdoc),
    #[command(subcommand, alias = "pl")]
    Pcblib(CmdPcblib),
}

#[derive(Debug, clap::Subcommand)]
pub enum CmdSchlib {
    List(LibListArgs),
}

#[derive(Debug, clap::Subcommand)]
pub enum CmdSchdoc {
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
    /// Match items with the given regular expression
    #[arg(short = 'E', long)]
    pub item_re: Vec<String>,

    /// List records for the items
    #[arg(short, long, default_value_t = false)]
    pub records: bool,

    /// Filter records (matched as regex)
    #[arg(short = 'f', long, value_delimiter = ',')]
    pub record_filter: Vec<String>,

    /// Only show these fields from the records (matched as regex)
    #[arg(short = 'd', long, value_delimiter = ',')]
    pub field_filter: Option<String>,
}
