use altium::pcb::{PcbLib};

const PCBLIB_EMPTY: &str = "tests/samples/pcblib/Empty.PcbLib";
const PCBLIB_SIMPLE: &str = "tests/samples/pcblib/Simple.PcbLib";
const ALL_PCBLIBS: &[&str] = &[PCBLIB_EMPTY,PCBLIB_SIMPLE];

#[test]
fn test_parse() {
    for path in ALL_PCBLIBS {
        let _pcblib = PcbLib::open(path).unwrap();
    }
}
