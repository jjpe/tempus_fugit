# Tempus Fugit

This is a Rust crate that operates around the concept of measuring the time it
takes to take some action.  Convenience is the name of the game here, and this
end it enables a dependent crate to do 2 things:

1. Measuring the wall-clock time of any expression in nanosecond[1] resolution:
    ```toml
    [dependencies]
    tempus_fugit = "0.4"
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

    The `measure` macro returns a tuple containing the result of executing
    an expression (in this case a block), as well as a `Measurement` which
    indicates how long the expression took to execute.


2. Displaying a `Measurement` in a human-readable fashion.
   There is a `Display` impl for `Measurement`, so this is as easy as
   formatting a value with e.g. `format!("{}", measurement)`.


[1] While the accounting is in nanosecond resolution, the actual resolution may
    be limited to courser granularity by the operating system.

In addition, the `Measurement` type has impls for `Ord` and `Eq`, which
makes comparison and ordering easy, as well as impls for de/serialization
through Serde.


## Documentation

The API docs are located [here](https://docs.rs/tempus_fugit/).
