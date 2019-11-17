# Tempus Fugit &emsp;  [![Latest Version]][crates.io] [![Rustc Version 1.26+]][rustc]

[Latest Version]: https://img.shields.io/crates/v/tempus_fugit.svg
[crates.io]: https://crates.io/crates/tempus_fugit
[Rustc Version 1.26+]: https://img.shields.io/badge/rustc-1.26+-lightgray.svg
[rustc]: https://nothing.here


**This is a Rust crate that operates around the concept of measuring the time it
takes to take some action.**

Convenience is the name of the game here, specifically by empowering a dependent
crate to do 2 things:

1. Measuring the wall-clock time of any expression in nanosecond[1] resolution:
    ```toml
    [dependencies]
    tempus_fugit = "0.8"
    ```

    ``` rust
    #[macro_use] extern crate tempus_fugit;

    use std::fs::File;
    use std::io::Read;
    use tempus_fugit::Measurement;

    fn main() {
        let (contents, measurement) = measure! {{
            let mut file = File::open("Cargo.lock")
                .expect("failed to open Cargo.lock");
            let mut contents = vec![];
            file.read_to_end(&mut contents)
                .expect("failed to read Cargo.lock");
            String::from_utf8(contents)
                .expect("failed to extract contents to String")
        }};

        println!("contents: {:?}", contents);
        println!("opening and reading file took {}", measurement);
    }
    ```

    The `measure!` macro returns a tuple containing the result of executing
    an expression (in this case a block), as well as a `Measurement` which
    indicates how long the expression took to execute.


2. Displaying a `Measurement` in a human-readable fashion.
   There is a `Display` impl for `Measurement`, so this is as easy as
   formatting a value with e.g. `format!("{}", measurement)`.


The `Measurement` type also has impls for `Ord` and `Eq`, which makes
comparison and sorting easy.

In addition, there is opt-in support for de/serialization through Serde.
This is activated by using the follwing in your crate's `Cargo.toml`:

``` toml
[dependencies]
tempus_fugit = { version = "0.8", features = ["enable_serde"] }

```



[1] While the accounting is in nanosecond resolution, the actual resolution may
    be limited to courser granularity by the operating system.


## Documentation

The API docs are located [here](https://docs.rs/tempus_fugit/).
