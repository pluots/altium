# Altium file format library for Rust

A tool to process Altium file types. Currently this tool is in alpha and only
has minimal components fully functioning (so please expect breaking changes!).

This is intended as a replacement for my original tool
[`PyAltium`](https://github.com/pluots/PyAltium)

## Project Progress

The goal of this project is to support most file types used by Altium. Reading
is a priority, writing will be implemented for some types. The status of
various file types is listed below:

|                          | Extension   | List Items | Display | Write | Documentation                         |
| ------------------------ | ----------- | ---------- | ------- | ----- | ------------------------------------- |
| Binary Schematic Library | .SchLib     | ✓          | Poorly  |       | Good                                  |
| Binary PCB Library       | .PcbLib     |            |         |       |                                       |
| Binary Schematic Doc     | .SchDoc     | ✓          |         |       |                                       |
| Binary PCB Doc           | .PcbDoc     |            |         |       |                                       |
| Draftsman Doc            | .PcbDwf     |            |         |       |                                       |
| PCB Project              | .PrjPcb     |            |         |       |                                       |
| Material Library         | .xml        |            | N/A     |       |                                       |
| Any templates            | Not Planned |            |         |       |                                       |

## Examples

See `altium/examples` for some sample usage. Example reading components in a
schematic library:

```rust
use altium::SchLib;

fn main() {
    let lib = SchLib::open("tests/samples/schlib/simple.SchLib").unwrap();

    // List all librefs stored in this schematic library
    for meta in lib.component_meta() {
        println!(
            "libref: {}, description: {}",
            meta.libref(),
            meta.description()
        );
    }

    // Get a single component by libref
    let mycomp = lib.get_component("Resistor - Standard").unwrap();

    // Write that image to a SVG file. Note that output is pretty buggy still.
    mycomp.save_svg("resistor.svg").unwrap();
}
```

## License

Currently, this is licensed under Apache 2.0.
