use parse::Parse;
use proc_macro::TokenStream;
use quote::quote;
use spanned::Spanned;
use syn::*;

#[derive(Clone)]
enum PField {
    Basic(Ident, Type),
    Tuple(u16, Type),
}

impl PField {
    fn write(&self, module: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match self {
            PField::Basic(i, _) => quote! { ::raad::#module::W::w(to, self.#i)?; },
            PField::Tuple(i, _) => quote! { ::raad::#module::W::w(to, self.#i)?; },
        }
    }
    fn read(&self, module: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match self {
            PField::Basic(i, _) => quote! { #i: ::raad::#module::R::r(from)?, },
            PField::Tuple(i, _) => {
                let i = proc_macro2::Literal::u16_unsuffixed(*i);
                quote! { #i: ::raad::#module::R::r(from)?, }
            }
        }
    }
}

impl Input {
    fn impl_block(
        &self,
        trt: proc_macro2::TokenStream,
        body: proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        let Self {
            generics: (params, generics),
            ident,
            ..
        } = self;
        let (intro_generics, fwd_generics, maybe_where_clause) = generics.split_for_impl();
        let binding = if maybe_where_clause.is_none() {
            quote!(where #(#params: ::raad::#trt,)*)
        } else {
            quote!(,#(#params: ::raad::#trt,)*)
        };
        quote! {
            impl #intro_generics ::raad::#trt for #ident #fwd_generics #maybe_where_clause #binding { #body }
        }
    }
}

struct Input {
    generics: (Vec<Ident>, Generics),
    ident: Ident,
    fields: Vec<PField>,
}

impl Parse for Input {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let DeriveInput {
            data,
            ident,
            generics,
            attrs,
            ..
        } = input.parse::<DeriveInput>()?;
        let params = generics.params.iter().filter_map(|x| match x {
            GenericParam::Type(x) => Some(x.ident.clone()),
            _ => None,
        });
        let Data::Struct(DataStruct { fields, .. }) = data else {
            return Err(input.error("only structs are supported for codegen"));
        };
        let fields = fields
            .iter()
            .zip(0..)
            .map(|(Field { ident, ty, .. }, i)| match ident {
                None => PField::Tuple(i, ty.clone()),
                Some(i) => PField::Basic(i.clone(), ty.clone()),
            })
            .collect();

        Ok(Input {
            fields,
            ident,
            generics: (params.collect(), generics),
        })
    }
}

#[proc_macro_derive(Write, attributes(raad))]
/// Types are written in order of declaration.
pub fn impl_write(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    let fields_le = input.fields.iter().map(|x| x.write(quote!(le)));
    let fields_be = input.fields.iter().map(|x| x.write(quote!(be)));

    let le_block = input.impl_block(
        quote!(le::Writable),
        quote! {
                fn _w(self, to: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
                    #(#fields_le)*
                    Ok(())
                }
        },
    );
    let be_block = input.impl_block(
        quote!(be::Writable),
        quote! {
                fn _w(self, to: &mut impl ::std::io::Write) -> ::std::io::Result<()> {
                    #(#fields_be)*
                    Ok(())
                }
        },
    );
    quote! { #le_block #be_block }.into()
}

#[proc_macro_derive(Read, attributes(raad))]
/// Types are read in order of declaration.
pub fn impl_read(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    let fields_le = input.fields.iter().map(|x| x.read(quote!(le)));
    let fields_be = input.fields.iter().map(|x| x.read(quote!(be)));

    let le_block = input.impl_block(
        quote!(le::Readable),
        quote! {
                fn r(from: &mut impl std::io::Read) -> ::std::io::Result<Self> {
                    Ok(Self { #(#fields_le)* })
                }
        },
    );
    let be_block = input.impl_block(
        quote!(be::Readable),
        quote! {
                fn r(from: &mut impl std::io::Read) -> ::std::io::Result<Self> {
                    Ok(Self { #(#fields_be)* })
                }
        },
    );
    quote! { #le_block #be_block }.into()
}
