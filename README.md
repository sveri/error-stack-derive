# Error Stack Derive

[https://docs.rs/crate/error-stack-derive/latest/error-stack-derive](![badge](https://img.shields.io/docsrs/error-stack-derive?label=documentation&logo=rust&style=flat-square))

A simple crate with a simple derive macro to make your error handling
workflow simpler than ever :)

Check out the [examples](examples) directory for, well, examples
and check out the docs for more information about the macro.

Or, here's one

```rust
use std::fs::read_to_string;

use error_stack::{IntoReport, ResultExt};
use error_stack_derive::ErrorStack;

#[derive(ErrorStack, Debug)]
#[error_message(&format!("Error occured with foo ({}, {})", self.bar, self.baz))]
struct FooError {
    bar: u8,
    baz: u8,
}

fn main() {
    let foo = read_to_string("foo.txt")
        .report()
        .change_context(FooError { bar: 0, baz: 1 });

    assert!(foo.is_err());
    let err = foo.err().unwrap();
    // Error occured with foo (0, 1)
    //              at examples/structs.rs:16:10
    // 
    // Caused by:
    //    0: No such file or directory (os error 2)
    //              at examples/structs.rs:15:10
    // FooError { bar: 0, baz: 1 }
    println!("{:?}\n{:?}", err, err.downcast_ref::<FooError>().unwrap())
}
```
