use altium::{sch::Component, SchLib};
use clap::Parser;
use cli::CmdSchlib;
use serde_json::{json, Value};

use crate::cli::Subcommand;

mod cli;

fn main() {
    let args = cli::Args::parse();

    match args.sub {
        Subcommand::Schlib(schlib_cmd) => handle_schlib_cmd(schlib_cmd),
        Subcommand::Pcblib(_pcblib_cmd) => {}
    };
}

fn handle_schlib_cmd(cmd: CmdSchlib) {
    match cmd {
        CmdSchlib::List(args) => {
            let lib = SchLib::open(&args.fname).unwrap();
            let cfg = PrintCfg {
                include_records: args.records,
                filter: &args.filter,
            };

            if args.item.is_empty() {
                lib.components()
                    .for_each(|comp| print_component(&comp, &cfg));
            } else {
                args.item
                    .iter()
                    .for_each(|name| print_component(&lib.get_component(name).unwrap(), &cfg))
            }
            // for comp in lib.components()
            //      {
            //     dbg!(comp);
            // }
        }
    }
}

struct PrintCfg<'a> {
    include_records: bool,
    filter: &'a [String],
}

/// Just use json to print the thing, it's easier
fn print_component(comp: &Component, cfg: &PrintCfg) {
    let val = if cfg.include_records {
        let records: Vec<_> = if cfg.filter.is_empty() {
            comp.records().collect()
        } else {
            comp.records()
                .filter(|rec| cfg.filter.iter().any(|filt| filt == rec.name()))
                .collect()
        };

        // Print full records
        json! {{
            comp.name(): {
                "records": records
            }
        }}
    } else {
        // Print only the name
        Value::String(comp.name().into())
    };

    let s = serde_json::to_string_pretty(&val).unwrap();
    println!("{s}");
}
