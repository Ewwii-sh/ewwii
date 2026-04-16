use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_attribute]
pub fn scan_prop(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let props_matches = if let Data::Enum(data_enum) = &input.data {
        data_enum
            .variants
            .iter()
            .filter_map(|v| match &v.fields {
                Fields::Named(fields) => {
                    for f in &fields.named {
                        if f.ident.as_ref().map(|id| id == "props").unwrap_or(false) {
                            let vname = &v.ident;
                            return Some(quote! {
                                #name::#vname { props, .. } => Some(props)
                            });
                        }
                    }
                    None
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    let expanded = quote! {
        #input

        impl #name {
            pub fn props(&self) -> Option<&Map> {
                match self {
                    #(#props_matches),*,
                    _ => None
                }
            }
        }
    };

    TokenStream::from(expanded)
}
