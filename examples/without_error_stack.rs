use error_stack_derive::ErrorStack;

#[derive(ErrorStack, Debug)]
#[error_message(&format!("An exception occured with foo: {}", self.0))]

struct FooError(String);
fn main() -> Result<(), FooError> {
    let contents = std::fs::read_to_string("foo.txt").map_err(|e| FooError(e.to_string()))?;

    println!("{contents}");

    Ok(())
}
