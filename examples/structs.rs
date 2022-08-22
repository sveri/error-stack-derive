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
    println!("{:?}\n{:?}", err, err.downcast_ref::<FooError>().unwrap())
}
