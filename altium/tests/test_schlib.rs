use std::cmp::min;
use std::io::prelude::*;
use std::{
    env,
    fs::{self, File},
    path::PathBuf,
};

use altium::sch::{storage::file_name, SchLib};

const SCHLIB_EMPTY: &str = "tests/samples/schlib/Empty.SchLib";
const SCHLIB_GRAPHIC: &str = "tests/samples/schlib/Graphic.SchLib";
const SCHLIB_GRAPHIC_SIMPLE: &str = "tests/samples/schlib/GraphicSimple.SchLib";
const SCHLIB_LONG_PART_NAMES: &str = "tests/samples/schlib/LongPartNames.SchLib";
const SCHLIB_LONG_PIN_DATA: &str = "tests/samples/schlib/LongPinData.SchLib";
const SCHLIB_SIMPLE: &str = "tests/samples/schlib/Simple.SchLib";

const ALL_SCHLIBS: &[&str] = &[
    SCHLIB_EMPTY,
    SCHLIB_GRAPHIC,
    SCHLIB_GRAPHIC_SIMPLE,
    SCHLIB_LONG_PART_NAMES,
    SCHLIB_LONG_PIN_DATA,
    SCHLIB_SIMPLE,
];

/// Name of a component in simple
const SIMPLE_COMP_NAME1: &str = "Mixed with shape and text";
const SIMPLE_COMP_NAME2: &str = "Pin_Properties";

#[test]
fn test_parse() {
    // Just test error free parsing
    let schlib = SchLib::open(SCHLIB_EMPTY).unwrap();
    println!("{schlib:#?}");
    let schlib = SchLib::open(SCHLIB_SIMPLE).unwrap();
    println!("{schlib:#?}");
}

#[test]
fn test_record() {
    let schlib = SchLib::open(SCHLIB_SIMPLE).unwrap();
    let comp = schlib.get_component(SIMPLE_COMP_NAME1).unwrap();
    println!("comp {SIMPLE_COMP_NAME1}:\n{comp:#?}");
    let comp = schlib.get_component(SIMPLE_COMP_NAME2).unwrap();
    println!("comp {SIMPLE_COMP_NAME2}:\n{comp:#?}");
}

#[test]
fn test_draw_single_svg() {
    // Only draw my favorite symbol
    let schlib = SchLib::open(SCHLIB_SIMPLE).unwrap();
    let mut out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    out_dir.extend(["test_output", "svg"]);
    fs::create_dir_all(&out_dir).unwrap();

    dbg!(schlib.storage());

    let comp = schlib.get_component("CombinedPinsRectGraphic").unwrap();
    let node = comp.svg();
    dbg!(&comp);
    let mut out_path = out_dir.clone();
    out_path.push(format!("{}.svg", comp.name().replace(' ', "_")));
    svg::save(&out_path, &node).unwrap();

    dbg!(schlib.storage());

    dbg!(altium::__private::num_unsupported_keys());
}

#[test]
fn test_draw_all_svgs() {
    for schlib_path in ALL_SCHLIBS {
        let schlib = SchLib::open(schlib_path).unwrap();

        let mut out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        out_dir.extend(["test_output", "svg"]);
        fs::create_dir_all(&out_dir).unwrap();

        for comp in schlib.components() {
            let node = comp.svg();
            let mut out_file = out_dir.clone();
            let fname = comp.name().replace(' ', "_");

            // Some have long names, truncate them
            out_file.push(format!("{}.svg", &fname[..min(40, fname.len())]));
            svg::save(&out_file, &node).unwrap();
            eprintln!("wrote {}", out_file.display());
        }
    }

    assert_eq!(altium::__private::num_unsupported_keys(), 0);
}

#[test]
fn test_storage() {
    let schlib = SchLib::open(SCHLIB_SIMPLE).unwrap();
    let mut out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    out_dir.extend(["test_output", "storage"]);
    fs::create_dir_all(&out_dir).unwrap();

    let storage = schlib.storage();

    for key in storage.keys() {
        let data = storage.get_data(key).unwrap();
        let out_file = out_dir.join(file_name(key));
        File::create(&out_file).unwrap().write_all(&data).unwrap();
        eprintln!("wrote {}", out_file.display());
    }
}
