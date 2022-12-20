use altium::PrjPcb;
use std::fs::read_to_string;

#[test]
fn test_file_ok() {
    PrjPcb::from_file("tests/files/example_proj.prjpcb").unwrap();
}
