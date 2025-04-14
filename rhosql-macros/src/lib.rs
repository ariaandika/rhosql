use proc_macro::TokenStream;

mod from_row;

/// Automatically implement `FromRow` for custom struct.
#[proc_macro_derive(FromRow, attributes(sql))]
pub fn from_row(input: TokenStream) -> TokenStream {
    match from_row::from_row(syn::parse_macro_input!(input as _)) {
        Ok(ok) => ok.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

