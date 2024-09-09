use proc_macro::TokenStream;
use quote::quote;
use spanned::Spanned;
use syn::*;
#[proc_macro_derive(Write, attributes(raad))]
/// Types are written in order of declaration.
pub fn impl_write(input: TokenStream) -> TokenStream {
    let DeriveInput {
        data,
        ident,
        vis,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let Data::Struct(DataStruct { fields, .. }) = data else {
        return syn::Error::new(vis.span(), "only structs are supported for codegen")
            .to_compile_error()
            .into();
    };
    let fields_le = fields
        .iter()
        .zip((0..).map(proc_macro2::Literal::usize_unsuffixed))
        .map(|(Field { ident, .. }, i)| match ident {
            None => quote! { ::raad::le::W::w(to, self.#i)?; },
            Some(i) => quote! { ::raad::le::W::w(to, self.#i)?; },
        });
    let fields_be = fields
        .iter()
        .zip((0..).map(proc_macro2::Literal::usize_unsuffixed))
        .map(|(Field { ident, .. }, i)| match ident {
            None => quote! { ::raad::be::W::w(to, self.#i)?; },
            Some(i) => quote! { ::raad::be::W::w(to, self.#i)?; },
        });
    quote! {
        impl ::raad::le::Writable for #ident {
            fn _w(self, to: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
                #(#fields_le)*
                Ok(())
            }
        }
        impl ::raad::be::Writable for #ident {
            fn _w(self, to: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
                #(#fields_be)*
                Ok(())
            }
        }
    }
    .into()
}

#[proc_macro_derive(Read, attributes(raad))]
/// Types are read in order of declaration.
pub fn impl_read(input: TokenStream) -> TokenStream {
    let DeriveInput {
        data,
        ident,
        vis,
        generics,
        ..
    } = parse_macro_input!(input as DeriveInput);
    let Data::Struct(DataStruct { fields, .. }) = data else {
        return syn::Error::new(vis.span(), "only structs are supported for codegen")
            .to_compile_error()
            .into();
    };
    let fields_le = fields
        .iter()
        .zip((0..).map(proc_macro2::Literal::usize_unsuffixed))
        .map(|(Field { ident, .. }, i)| match ident {
            None => quote! { #i: ::raad::le::R::r(from)?, },
            Some(i) => quote! { #i: ::raad::le::R::r(from)?, },
        });
    let fields_be = fields
        .iter()
        .zip((0..).map(proc_macro2::Literal::usize_unsuffixed))
        .map(|(Field { ident, .. }, i)| match ident {
            None => quote! { #i: ::raad::be::R::r(from)?, },
            Some(i) => quote! { #i: ::raad::be::R::r(from)?, },
        });
    quote! {
        impl ::raad::le::Readable for #ident {
            fn r(from: &mut impl std::io::Read) -> ::std::io::Result<Self> {
                Ok(Self { #(#fields_le)* })
            }
        }
        impl ::raad::be::Readable for #ident {
            fn r(from: &mut impl ::std::io::Read) -> ::std::io::Result<Self> {
                Ok(Self { #(#fields_be)* })
            }
        }
    }
    .into()
}
