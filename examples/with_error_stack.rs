use error_stack::{IntoReport, Result, ResultExt};
use error_stack_derive::ErrorStack;

#[derive(ErrorStack, Debug)]
#[error_message("An exception occured with foo")]
struct FooError;

fn main() -> Result<(), FooError> {
    let contents = std::fs::read_to_string("foo.txt")
        .report()
        .change_context(FooError)
        .attach_printable("Unable to read foo.txt file")?;

    println!("{contents}");

    Ok(())
}
