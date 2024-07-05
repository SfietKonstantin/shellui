#![allow(clippy::manual_unwrap_or_default)]
use darling::ast::Data;
use darling::util::Ignored;
use darling::{FromDeriveInput, FromField};
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Generics, Ident, Index, Type};

#[proc_macro_derive(ObjectFormatter, attributes(object_formatter))]
pub fn display_cli(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let raw = parse_macro_input!(input as DeriveInput);
    let input = FormatterInput::from_derive_input(&raw);
    let expanded = match input {
        Ok(input) => {
            let headers = implement_headers(&input);
            //let headers_with_mode = implement_headers(&input, implement_header_with_mode);
            let format_value = implement_format_value(&input);

            let name = input.ident;
            let type_params = input.generics.type_params();
            let (_, ty_generics, where_clause) = input.generics.split_for_impl();

            quote! {
                impl <#(#type_params,)*> shellui::format::ObjectFormatter for #name #ty_generics #where_clause {
                    type Header = &'static str;
                    type Mode = &'static str;

                    fn headers(mode: Option<Self::Mode>) -> Vec<Self::Header> {
                        #headers
                    }

                    fn format_value(&self, mode: Option<Self::Mode>, header: &Self::Header) -> String {
                        #format_value
                    }
                }
            }
        }
        Err(error) => {
            let message = error.to_string();
            quote_spanned! { raw.ident.span() => compile_error!(#message); }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_any))]
struct FormatterInput {
    ident: Ident,
    generics: Generics,
    data: Data<Ignored, FormatterField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(object_formatter))]
struct FormatterField {
    ident: Option<Ident>,
    ty: Type,

    #[darling(default)]
    inline: bool,
    #[darling(default)]
    header: Option<String>,
    #[darling(default)]
    mode: Option<String>,
}

fn implement_headers(input: &FormatterInput) -> TokenStream {
    let data = input.data.as_ref();
    let struct_data = data.take_struct();
    let headers = struct_data
        .iter()
        .flat_map(|i| i.fields.iter().copied())
        .map(implement_header);
    quote! {
        let mut headers = Vec::new();
        #(#headers)*
        headers
    }
}

fn implement_header(field: &FormatterField) -> TokenStream {
    match (&field.inline, &field.header, &field.mode) {
        (true, None, None) => {
            let ty = &field.ty;
            quote! {
                for header in #ty::headers(mode.clone()) {
                    headers.push(header);
                }
            }
        }
        (false, Some(header), None) => {
            quote! {
                headers.push(#header);
            }
        }
        (false, Some(header), Some(mode)) => {
            quote! {
                if mode == Some(#mode) {
                    headers.push(#header);
                }
            }
        }
        (false, None, None) => {
            quote! {}
        }
        _ => {
            quote_spanned! { field.ident.span() => compile_error!("Invalid object_formatter attribute"); }
        }
    }
}

fn implement_format_value(input: &FormatterInput) -> TokenStream {
    let data = input.data.as_ref();
    let struct_data = data.take_struct();
    let elements = struct_data
        .iter()
        .flat_map(|i| i.fields.iter().copied().enumerate())
        .filter_map(|(index, field)| implement_format_single_value(index, field))
        .collect::<Vec<_>>();

    if elements.is_empty() {
        quote! { String::new() }
    } else {
        let else_keyword = quote! { else };
        let elements =
            Itertools::intersperse(elements.into_iter(), else_keyword).collect::<Vec<_>>();

        quote! {
            #(#elements)*
            else {
                String::new()
            }
        }
    }
}

fn implement_format_single_value(index: usize, field: &FormatterField) -> Option<TokenStream> {
    match (&field.inline, &field.header, &field.mode) {
        (true, None, None) => {
            let ty = &field.ty;
            let access = format_access(index, field);
            let value = quote! {
                 if #ty::headers(mode.clone()).contains(header) {
                    #access.format_value(mode.clone(), header)
                }
            };
            Some(value)
        }
        (false, Some(header), _) => {
            let access = format_access(index, field);
            let value = quote! {
                if *header == #header {
                    shellui::format::FormatField::format_field(&#access)
                }
            };
            Some(value)
        }
        _ => None,
    }
}

fn format_access(index: usize, field: &FormatterField) -> TokenStream {
    if let Some(ident) = &field.ident {
        let ident = ident.clone();
        quote! {
            self.#ident
        }
    } else {
        let index = Index {
            index: index as u32,
            span: Span::call_site(),
        };
        quote! {
            self.#index
        }
    }
}
