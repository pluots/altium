use std::{
    fs::{self, File},
    io,
    path::{self, PathBuf},
};

use altium::{
    sch::{Component, ComponentMeta, SchRecord},
    SchDoc,
    SchLib,
};
use anyhow::Context;
use cap_std::fs::{Dir, OpenOptions};
use cfb::CompoundFile;
use clap::Parser;
use cli::{CmdSchdoc, CmdSchlib};
use regex::Regex;
use serde_json::{json, Value};

use crate::cli::{CfbArgs, CmdCfb, Subcommand};

mod cli;

fn main() {
    let args = cli::Args::parse();

    match args.sub {
        Subcommand::Schlib(schlib_cmd) => handle_schlib_cmd(schlib_cmd),
        Subcommand::Schdoc(schdoc_cmd) => handle_schdoc_cmd(schdoc_cmd),
        Subcommand::Pcblib(_pcblib_cmd) => {
            unimplemented!("not yet implemented")
        }
        Subcommand::Cfb(cfb_cmd) => handle_cfb_cmd(cfb_cmd),
    };
}

fn handle_schlib_cmd(cmd: CmdSchlib) {
    match cmd {
        CmdSchlib::List(args) => {
            let lib = SchLib::open(&args.fname).unwrap();
            let cfg = PrintCfg::new(
                args.records,
                &args.record_filter,
                args.field_filter.as_deref(),
            );

            if args.item.is_empty() && args.item_re.is_empty() {
                // Print all components
                lib.components()
                    .for_each(|comp| print_component(&comp, &cfg));
            } else {
                let re_vec: Vec<_> = args.item_re.iter().map(|re| to_re(re)).collect();

                lib.component_meta()
                    .iter()
                    .map(ComponentMeta::libref)
                    .filter(|libref| {
                        args.item
                            .iter()
                            .any(|item_name| item_name.eq_ignore_ascii_case(libref))
                            || re_vec.iter().any(|re| re.is_match(libref))
                    })
                    .for_each(|libref| print_component(&lib.get_component(libref).unwrap(), &cfg));
            }
        }
    }
}

fn handle_schdoc_cmd(cmd: CmdSchdoc) {
    match cmd {
        CmdSchdoc::List(args) => {
            let doc = SchDoc::open(&args.fname).unwrap();
            let cfg = PrintCfg::new(
                args.records,
                &args.record_filter,
                args.field_filter.as_deref(),
            );

            print_records(&args.fname.display().to_string(), doc.records(), &cfg)
            // doc.records()
        }
    }
}

fn to_re(re: &str) -> Regex {
    regex::RegexBuilder::new(re)
        .case_insensitive(true)
        .build()
        .unwrap()
}

struct PrintCfg {
    include_records: bool,
    record_filter: Vec<Regex>,
    field_filter: Option<Regex>,
}

impl PrintCfg {
    fn new(include_records: bool, record_filter: &[String], field_filter: Option<&str>) -> Self {
        Self {
            include_records,
            record_filter: record_filter.iter().map(|re| to_re(re)).collect(),
            field_filter: field_filter.map(to_re),
        }
    }

    fn matches_rec_filter(&self, s: &str) -> bool {
        self.record_filter.iter().any(|filt| filt.is_match(s))
    }
}

fn print_component(comp: &Component, cfg: &PrintCfg) {
    print_records(comp.name(), comp.records().iter(), cfg);
}

/// Just use json to print the thing, it's easier
fn print_records<'a, I>(name: &str, records: I, cfg: &PrintCfg)
where
    I: Iterator<Item = &'a SchRecord>,
{
    let mut records: Box<dyn Iterator<Item = &SchRecord>> = Box::new(records);

    let val = if cfg.include_records {
        if !cfg.record_filter.is_empty() {
            records = Box::new(records.filter(|rec| cfg.matches_rec_filter(rec.name())))
        };

        // let mut records = serde_json::to_value(records.collect::<Vec<_>>()).unwrap();
        let mut rec_vec = Vec::new();

        for rec in records {
            let mut val = serde_json::to_value(rec).unwrap();
            let rec_map = val.as_object_mut().unwrap();

            if let Some(re) = &cfg.field_filter {
                // Remove undesired fields
                println!("{:?}", rec_map);
                rec_map.retain(|k, _v| re.is_match(k));
            }

            if !rec_map.is_empty() {
                rec_vec.push(val);
            }
        }

        // Print full records
        json! {{ name: { "records": rec_vec } }}
    } else {
        // Print only the name
        Value::String(name.into())
    };

    let s = serde_json::to_string_pretty(&val).unwrap();
    println!("{s}");
}

fn handle_cfb_cmd(cmd: CmdCfb) {
    match cmd {
        CmdCfb::Extract(args) => {
            let CfbArgs { file, dest: dpath } = args;

            let mut f = File::open(&file)
                .context("failed to open file")
                .and_then(|f| CompoundFile::open(f).context("could not read file as CFB"))
                .with_context(|| format!("opening `{}`", file.display()))
                .unwrap();

            fs::create_dir_all(&dpath)
                .unwrap_or_else(|e| panic!("failed to create or open `{}`: {e}", dpath.display()));
            let dst = Dir::open_ambient_dir(&dpath, cap_std::ambient_authority())
                .unwrap_or_else(|e| panic!("failed to open `{}`: {e}", dpath.display()));

            let entries = f.walk().collect::<Vec<_>>();

            for entry in entries {
                let orig_path = entry.path();

                // Paths are absolute, strip the first component to make them relative.
                let mut components = orig_path.components();
                assert_eq!(
                    components.next(),
                    Some(path::Component::RootDir),
                    "sanity check failed"
                );
                let p: PathBuf = components.collect();

                if entry.is_root() {
                    // Nothing to do
                } else if entry.is_storage() {
                    dst.create_dir_all(&p).unwrap_or_else(|e| {
                        panic!("failed to create directory for storage {p:?} at {dpath:?}: {e}")
                    });
                } else if entry.is_stream() {
                    let mut stream = f.open_stream(orig_path).unwrap();
                    let mut stream_file = dst
                        .open_with(&p, OpenOptions::new().create_new(true).write(true))
                        .unwrap_or_else(|e| {
                            panic!("failed to create file for stream {p:?} at {dpath:?}: {e}")
                        });
                    io::copy(&mut stream, &mut stream_file).unwrap_or_else(|e| {
                        panic!("failed to write file for stream {p:?} at {dpath:?}: {e}")
                    });
                } else {
                    panic!("unknown entry kind at {}", entry.path().display());
                }
            }
        }
    }
}
