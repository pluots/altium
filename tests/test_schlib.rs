use altium::schlib::SchLib;

const SCHLIB_EMPTY: &str = "tests/samples/schlib/Empty.SchLib";
const SCHLIB1: &str = "tests/samples/schlib/SchLib1.SchLib";

#[test]
fn test_parse() {
    // Just test error free parsing
    SchLib::open(SCHLIB_EMPTY).unwrap();
    SchLib::open(SCHLIB1).unwrap();
}
