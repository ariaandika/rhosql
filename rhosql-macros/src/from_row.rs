use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    token::{Brace, Paren},
    *,
};

macro_rules! error {
    ($($tt:tt)*) => {
        return Err(syn::Error::new(proc_macro2::Span::call_site(), format!($($tt)*)))
    };
}

pub fn from_row(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput { attrs: _, vis: _, ident, generics, data } = input;
    let Data::Struct(data) = data else {
        error!("only struct are currently supported")
    };

    let mut output = quote! {};

    match data.fields {
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let body = (0..unnamed.len())
                .map(Index::from)
                .map(|i|quote! { row.try_decode(#i)?, });
            Paren::default().surround(&mut output, |e|e.extend(body));
        },
        Fields::Named(FieldsNamed { named, .. }) => {
            let body = named
                .into_iter()
                .map(|e|e.ident.unwrap())
                .zip(0..)
                .map(|(e,i)|(e,Index::from(i)))
                .map(|(id,i)|quote! { #id: row.try_decode(#i)?, });
            Brace::default().surround(&mut output, |e|e.extend(body));
        }
        Fields::Unit => {}
    };

    let (g1, g2, g3) = generics.split_for_impl();

    Ok(quote! {
        impl #g1 ::rhosql::FromRow for #ident #g2 #g3 {
            fn from_row(row: ::rhosql::Row) -> ::rhosql::Result<Self> {
                Ok(Self #output)
            }
        }
    })
}

