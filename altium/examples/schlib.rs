use altium::SchLib;

fn main() {
    let lib = SchLib::open("path/to/mylib.schlib").unwrap();

    // List all librefs stored in this schematic library
    for meta in lib.component_meta() {
        println!(
            "libref: {}, description: {}",
            meta.libref(),
            meta.description()
        );
    }

    // Get a single component by libref
    let mycomp = lib.get_component("Resistor - standard").unwrap();

    // Write that image to a SVG file. Note that output is pretty buggy still.
    mycomp.save_svg("resistor.svg").unwrap();
}
