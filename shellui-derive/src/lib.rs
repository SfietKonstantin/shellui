use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Expr, Field, Fields, Lit};

#[proc_macro_derive(ObjectFormatter, attributes(header))]
pub fn display_cli(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let type_params = input.generics.type_params();
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    let mapping = build_mapping(input.data);
    let headers = implement_headers(&mapping);
    let format_value = implement_format_value(&mapping);

    let expanded = quote! {
        impl <#(#type_params,)*> shellui::format::ObjectFormatter for #name #ty_generics #where_clause {
            type Header = &'static str;

            fn headers() -> Vec<Self::Header> {
                #headers
            }

            fn format_value(&self, header: &Self::Header) -> String {
                #format_value
            }
        }
    };
    TokenStream::from(expanded)
}

enum HeaderMapping {
    InvalidHeaders(Vec<Attribute>),
    TooManyHeaders(Field),
    Mapping {
        name: String,
        index: usize,
        field: Field,
    },
    InlineMapping {
        index: usize,
        field: Field,
    },
}

fn build_mapping(data: Data) -> Result<Vec<HeaderMapping>, Span> {
    match data {
        Data::Enum(data) => Err(data.enum_token.span()),
        Data::Struct(data) => {
            let fields = match data.fields {
                Fields::Named(fields) => fields.named.iter().cloned().collect::<Vec<_>>(),
                Fields::Unnamed(fields) => fields.unnamed.iter().cloned().collect::<Vec<_>>(),
                Fields::Unit => Vec::new(),
            };

            let mapping = fields
                .into_iter()
                .enumerate()
                .filter_map(|(i, field)| build_field_mapping(i, field))
                .collect();
            Ok(mapping)
        }
        Data::Union(data) => Err(data.union_token.span()),
    }
}

fn build_field_mapping(index: usize, field: Field) -> Option<HeaderMapping> {
    let (headers, failed) = field
        .attrs
        .clone()
        .into_iter()
        .filter_map(parse_attribute)
        .fold(
            (Vec::new(), Vec::new()),
            |(mut headers, mut failed), result| {
                match result {
                    Ok(header) => headers.push(header),
                    Err(attribute) => failed.push(attribute),
                };
                (headers, failed)
            },
        );

    if failed.is_empty() {
        let mut iter = headers.into_iter();
        let first = iter.next();
        let second = iter.next();

        match (first, second) {
            (None, None) => None,
            (Some(header), None) => match header {
                Header::Mapping(name) => Some(HeaderMapping::Mapping { name, index, field }),
                Header::InlineMapping => Some(HeaderMapping::InlineMapping { index, field }),
            },
            _ => Some(HeaderMapping::TooManyHeaders(field)),
        }
    } else {
        Some(HeaderMapping::InvalidHeaders(failed))
    }
}

fn implement_headers(mapping: &Result<Vec<HeaderMapping>, Span>) -> impl ToTokens {
    match mapping {
        Ok(mapping) => {
            let headers = mapping.iter().map(implement_header);
            quote! {
                let mut headers = Vec::new();
                #(#headers)*
                headers
            }
        }
        Err(span) => {
            let span = *span;
            quote_spanned! { span => compile_error!("Unsupported type"); }
        }
    }
}

fn implement_header(mapping: &HeaderMapping) -> impl ToTokens {
    match mapping {
        HeaderMapping::InvalidHeaders(attributes) => {
            let errors = attributes.iter()
                .map(|attr| {
                    quote_spanned! { attr.bracket_token.span => compile_error!("Unsupported header attribute"); }
                });
            quote! {
                #(#errors)*
            }
        }
        HeaderMapping::TooManyHeaders(field) => {
            quote_spanned! { field.ident.span() => compile_error!("Too many header attributes"); }
        }
        HeaderMapping::Mapping { name, .. } => {
            quote! {
                headers.push(#name);
            }
        }
        HeaderMapping::InlineMapping { field, .. } => {
            let ty = &field.ty;
            quote! {
                for header in #ty::headers() {
                    headers.push(header);
                }
            }
        }
    }
}

fn implement_format_value(mapping: &Result<Vec<HeaderMapping>, Span>) -> impl ToTokens {
    match mapping {
        Ok(mapping) => {
            let mut iter = mapping.iter();
            if let Some(first) = iter.next() {
                let first = implement_format_single_value(first);
                let rest = iter.map(|mapping| {
                    let access = implement_format_single_value(mapping);
                    quote! {
                        else #access
                    }
                });

                quote! {
                    #first
                    #(#rest)*
                    else {
                        String::new()
                    }
                }
            } else {
                quote! { String::new() }
            }
        }
        Err(_) => {
            quote! {}
        }
    }
}

fn implement_format_single_value(mapping: &HeaderMapping) -> impl ToTokens {
    match mapping {
        HeaderMapping::InvalidHeaders(_) | HeaderMapping::TooManyHeaders(_) => quote! {},
        HeaderMapping::Mapping { name, index, field } => {
            let access = format_access(*index, field);
            quote! {
                if *header == #name {
                    #access.to_string()
                }
            }
        }
        HeaderMapping::InlineMapping { index, field } => {
            let ty = &field.ty;
            let access = format_access(*index, field);
            quote! {
                if #ty::headers().contains(header) {
                    #access.format_value(header)
                }
            }
        }
    }
}

fn format_access(index: usize, field: &Field) -> impl ToTokens {
    if let Some(ident) = &field.ident {
        let ident = ident.clone();
        quote! {
            self.#ident
        }
    } else {
        quote! {
            self.#index
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Header {
    Mapping(String),
    InlineMapping,
}

fn parse_attribute(attribute: Attribute) -> Option<Result<Header, Attribute>> {
    if attribute.path().is_ident("header") {
        let content = attribute.parse_args::<Expr>().ok();
        match content {
            Some(Expr::Lit(lit)) => match lit.lit {
                Lit::Str(value) => Some(Ok(Header::Mapping(value.value()))),
                _ => Some(Err(attribute)),
            },
            Some(Expr::Path(path)) => {
                if path.path.is_ident("inline") {
                    Some(Ok(Header::InlineMapping))
                } else {
                    Some(Err(attribute))
                }
            }
            _ => Some(Err(attribute)),
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse2, Variant};

    #[test]
    fn test_attr() {
        let attr = quote! {
            #[header("test")]
            Test
        };

        let input: Variant = parse2(attr).unwrap();
        let values = parse_attribute(input.attrs.get(0).cloned().unwrap())
            .unwrap()
            .unwrap();
        let expected = Header::Mapping("test".to_string());
        assert_eq!(values, expected);
    }

    #[test]
    fn test_attr_inline() {
        let attr = quote! {
            #[header(inline)]
            Test
        };

        let input: Variant = parse2(attr).unwrap();
        let values = parse_attribute(input.attrs.get(0).cloned().unwrap())
            .unwrap()
            .unwrap();
        let expected = Header::InlineMapping;
        assert_eq!(values, expected);
    }
}
