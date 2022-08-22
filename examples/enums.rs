use std::collections::HashMap;

use error_stack::{IntoReport, Report, ResultExt};
use error_stack_derive::ErrorStack;
use serde_json::{from_str, to_string};

#[derive(ErrorStack, Debug)]
// #[error_message("Default error message")]
enum MainError<T>
where
    T: std::fmt::Debug,
{
    #[error_message(&format!("Couldn't serialize data: {:?}", unnamed0))]
    SerializeError(T),
    #[error_message("Couldn't deserialize data")]
    DeserializeError,
    #[error_message(inner)]
    FooError {
        inner: &'static str,
    },
    // Will have default message
    BarError,
}

fn main() {
    let from_string = from_str::<String>("")
        .report()
        .change_context(MainError::<()>::DeserializeError)
        .attach_printable_lazy(|| r#"Data: """#);

    assert!(from_string.is_err());
    println!("{:#?}", from_string.err().unwrap());

    let map = HashMap::from([(false, "non string key")]);

    let to_string = to_string(&map)
        .report()
        .change_context(MainError::SerializeError(map));

    assert!(to_string.is_err());
    println!("{:#?}", to_string.err().unwrap());

    println!(
        "{:#?}",
        Report::new(MainError::<()>::FooError { inner: "hello" })
    );

    println!("{:#?}", Report::new(MainError::<()>::BarError));
}
