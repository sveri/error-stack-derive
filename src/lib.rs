//! The heart of this crate is [`ErrorStack`],
//! its a derive macro to make generating enums and structs compatible
//! with [error_stack](https://docs.rs/error-stack/latest/error_stack/).
//! Even though the sole purpose is provind a better DX with `error_stack`
//! this derive macro can actually be used with any other error system
//! since the crate itself doesn't depend on `error_stack` all it does
//! is makes [`std::error::Error`] and [`std::fmt::Display`] implementations
//! easy.
//!
//! ## Usage with and without `error_stack`
//!
//! ### With
//!
//! > Note:
//! > As of right-now `no-std` is not supported
//!
//! With `error_stack` you get the `Report` and their fancy attachments,
//! context, frames, etc. features, which to say the least are
//! pretty cool and helpful for error handling & debugging.
//!
//! ```
//! use error_stack::{IntoReport, Result, ResultExt};
//! use error_stack_derive::ErrorStack;
//!
//! #[derive(ErrorStack, Debug)]
//! #[error_message("An exception occured in foo")]
//! struct FooError;
//!
//! fn main() -> Result<(), FooError> {
//!     let contents = std::fs::read_to_string("foo.txt")
//!         .report()
//!         .change_context(FooError)
//!         .attach_printable("Unable to read foo.txt file")?;
//!
//!     println!("{contents}");
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Without
//!
//! Ofcourse this crate doesn't enforce the usage of `error_stack`
//! infact you can use it with any other error handling crate,
//! just like this
//!
//! ```
//! use error_stack_derive::ErrorStack;
//!
//! #[derive(ErrorStack, Debug)]
//! #[error_message(&format!("An exception occured with foo: {}", self.0))]
//!
//! struct FooError(String);
//! fn main() -> Result<(), FooError> {
//!     let contents = std::fs::read_to_string("foo.txt").map_err(|e| FooError(e.to_string()))?;
//!
//!     println!("{contents}");
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Looking into the expansion
//!
//! This crate, specifically the derive macro, does 2 things, <br />
//! one, implements [`std::error::Error`] <br />
//! two, implements [`std::fmt::Display`] <br />
//! you can derive a struct or an enum, the trait impl are pretty
//! simple
//!
//! For a struct:
//!
//! ```
//! // #[derive(error_stack_derive::ErrorStack, Debug)]
//! // #[error_message("An exception occured in foo")]
//! // struct FooError;
//!
//! #[derive(Debug)]
//! struct FooError;
//!
//! impl std::fmt::Display for FooError {
//!     fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         fmt.write_str("An exception occured in foo")
//!     }
//! }
//!
//! impl std::error::Error for FooError {}
//! ```
//!
//! For an enum:
//!
//! ```
//! // #[derive(error_stack_derive::ErrorStack, Debug)]
//! // enum FooErrors {
//! //  #[error_message("An exception in bar")]
//! //  BarError,
//! //  #[error_message(&format!("Error in baz ({unnamed0})"))]
//! //  BazError(String),
//! //  #[error_message(&format!("Error in qux ({start}, {end})"))]
//! //  QuxError {
//! //      start: u64,
//! //      end: u64,
//! //  }
//! // };
//!
//! #[derive(Debug)]
//! enum FooErrors {
//!  BarError,
//!  BazError(String),
//!  QuxError {
//!      start: u64,
//!      end: u64,
//!  }
//! };
//!
//! impl std::fmt::Display for FooErrors {
//!     fn fmt(&self, _____fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             Self::BarError => _____fmt.write_str(&format!("[{name}] An error occured; {:?}", name = "FooErrors", self)),
//!             Self::BazError(unnamed0) => _____fmt.write_str(&format!("Error in baz ({unnamed0})")),
//!             Self::QuxError { start, end } => _____fmt.write_str(&format!("Error in qux ({start}, {end})")),
//!         }
//!     }
//! }
//!
//! impl std::error::Error for FooErrors {}
//! ```
//!
//! As you can see its a pretty simple macro but definitely helps when
//! you have a large code base and error handling definitely becomes dreadful.
//! Read up the doc comments of [`ErrorStack`] for more information.
//!
use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{parse, parse_str, Attribute, Data, DataEnum, DeriveInput, Fields, Generics, Ident};

/// A derive-macro to easily create enums and structs compatible with
/// error_stack. You can use a struct or an enum with it
///
///
/// ## Panic
///
/// - When input cannot be passed as [`syn::DeriveInput`]
/// - When the derive data is not one of [`syn::Data::Enum`] or
/// [`syn::Data::Struct`]
///
///
/// ## Usage
///
/// ```
/// use error_stack_derive::ErrorStack;
///
/// #[derive(ErrorStack, Debug)]
/// // error_message tokens can be any token stream as long as it evaluates
/// // to a &str
/// #[error_message("An error occured in Foo")]
/// struct FooError;
///
/// #[derive(ErrorStack, Debug)]
/// // The tokens are passed to the [`std::fmt::Formatter::write_str`]
/// // method of the [`std::fmt::Formatter`] in the automatically
/// // implemented Display impl. Passing an error message is mandatory
/// // for structs while its not for enums
/// // So you can do this too!
/// #[error_message(&format!("An internal error occured: {}", self.0))]
/// struct InternalError<A>(pub A)
/// where
///     A: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static;
///
///
/// // And ofcourse enums are supported too
/// #[derive(ErrorStack, Debug)]
/// // This is the default error message, this is used when a variant
/// // doesn't have a dedicated error message
/// // When a default error message is not specified and an enum doesn't
/// // have a dedicated message,
/// // `&format!("[{name}] An error occured; {:?}", name = #struct_name, self)` is passed to
/// // [`std::fmt::Formatter::write_str`]
/// #[error_message("Default error message")]
/// enum EncoderError {
///     // For struct variants the name of the fields are left unchanged
///     // but for tuple variants they are named `unnamed{pos}`
///     #[error_message(&format!("Couldn't serialize data: {:?}", unnamed0))]
///     SerializeError(String),
///     DeserializeError,
/// }
/// ```
#[proc_macro_derive(ErrorStack, attributes(error_message))]
pub fn error(tokens: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        vis: _,
        ident,
        generics,
        data,
    } = parse(tokens).expect("derive input");

    let ast = match data {
        Data::Enum(data) => create_enum(attrs, ident, generics, data),
        Data::Struct(_) => create_struct(attrs, ident, generics),
        _ => panic!("#[derive(ErrorStack)] only supports structs and enums"),
    };

    ast.into()
}

fn create_enum(
    attrs: Vec<Attribute>,
    ident: Ident,
    Generics {
        lt_token,
        params,
        gt_token,
        where_clause,
    }: Generics,
    DataEnum {
        enum_token: _,
        brace_token: _,
        variants,
    }: DataEnum,
) -> TokenStream {
    let message = match attrs
        .iter()
        .find(|attr| attr.path.is_ident("error_message"))
    {
        Some(attr) => attr.tokens.to_owned(),
        None => {
            let name = syn::LitStr::new(&ident.to_string(), ident.span());
            quote!(&format!(
                "[{name}] An error occured; {:?}",
                self,
                name = #name,
            ))
        }
    };

    let match_arms = {
        let mut tmp = quote!();
        tmp.append_all(variants.iter().filter_map(|variant| {
            let ident = variant.ident.to_owned();
            let message = variant.attrs.iter().find_map(|attr| {
                if attr.path.is_ident("error_message") {
                    return Some(attr.tokens.to_owned());
                }
                None
            });

            let additional = match variant.fields {
                Fields::Named(ref named) => {
                    let mut tmp = quote!();
                    tmp.append_all(named.named.iter().map(|field| {
                        let ident = field.ident.to_owned();
                        quote! {
                            #ident ,
                        }
                    }));
                    quote! {{
                        #tmp
                    }}
                }
                Fields::Unnamed(ref unnamed) => {
                    let mut tmp = quote!();
                    tmp.append_all(unnamed.unnamed.iter().enumerate().map(|(pos, _)| {
                        let ident: Ident = parse_str(&format!("unnamed{pos}")).unwrap();
                        quote! {
                            #ident ,
                        }
                    }));
                    quote! {(#tmp)}
                }
                Fields::Unit => quote!(),
            };

            match message {
                Some(tokens) => Some(quote! {
                    Self::#ident #additional => _____fmt.write_str(#tokens),
                }),
                None => None,
            }
        }));
        tmp
    };

    quote! {
        impl #lt_token #params #gt_token std::fmt::Display for #ident #lt_token #params #gt_token #where_clause {
            fn fmt(&self, _____fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #[allow(unused_parens)]
                match self {
                    #match_arms
                    _ => _____fmt.write_str(#message)
                }
            }
        }

        impl #lt_token #params #gt_token std::error::Error for #ident #lt_token #params #gt_token #where_clause {}
    }
    .into()
}

fn create_struct(
    attrs: Vec<Attribute>,
    ident: Ident,
    Generics {
        lt_token,
        params,
        gt_token,
        where_clause,
    }: Generics,
) -> TokenStream {
    let message = attrs
        .iter()
        .find(|attr| attr.path.is_ident("error_message"))
        .expect("expected error message")
        .tokens
        .to_owned();

    quote! {
        impl #lt_token #params #gt_token std::fmt::Display for #ident #lt_token #params #gt_token #where_clause {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #[allow(unused_parens)]
                fmt.write_str(#message)
            }
        }

        impl #lt_token #params #gt_token std::error::Error for #ident #lt_token #params #gt_token #where_clause {}
    }
    .into()
}
