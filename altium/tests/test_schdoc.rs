include!("include_test_util.rs");

use altium::sch::SchDoc;

const SCHDOC_SIMPLE: &str = "tests/samples/schdoc/simple.SchDoc";

#[test]
fn test_parse() {
    test_init_once();
    // Just test error free parsing
    let schdoc = SchDoc::open(SCHDOC_SIMPLE).unwrap();
    println!("{schdoc:#?}");
}
