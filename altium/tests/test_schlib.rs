use std::{env, fs, path::PathBuf};

use altium::sch::SchLib;

const SCHLIB_EMPTY: &str = "tests/samples/schlib/Empty.SchLib";
const SCHLIB_SIMPLE: &str = "tests/samples/schlib/Simple.SchLib";
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
fn test_draw_svgs() {
    let schlib = SchLib::open(SCHLIB_SIMPLE).unwrap();
    let mut out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    out_dir.extend(["test_output", "svg"]);
    fs::create_dir_all(&out_dir).unwrap();

    let comp = schlib.get_component("CombinedPinsRectGraphic").unwrap();
    let node = comp.svg();
    let mut out_path = out_dir.clone();
    out_path.push(format!("{}.svg", comp.name().replace(' ', "_")));
    svg::save(&out_path, &node).unwrap();

    dbg!(altium::__private::num_unsupported_keys());
}

#[test]
fn test_draw_all_svgs() {
    let schlib = SchLib::open(SCHLIB_SIMPLE).unwrap();
    let mut out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    out_dir.extend(["test_output", "svg"]);
    fs::create_dir_all(&out_dir).unwrap();

    for comp in schlib.components() {
        let node = comp.svg();
        let mut out_path = out_dir.clone();
        out_path.push(format!("{}.svg", comp.name().replace(' ', "_")));
        svg::save(&out_path, &node).unwrap();
    }

    dbg!(altium::__private::num_unsupported_keys());
}
