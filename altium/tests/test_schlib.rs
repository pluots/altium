use altium::schlib::SchLib;

const SCHLIB_EMPTY: &str = "tests/samples/schlib/Empty.SchLib";
const SCHLIB_SIMPLE: &str = "tests/samples/schlib/Simple.SchLib";

#[test]
fn test_parse() {
    // Just test error free parsing
    SchLib::open(SCHLIB_EMPTY).unwrap();
    SchLib::open(SCHLIB_SIMPLE).unwrap();
}
