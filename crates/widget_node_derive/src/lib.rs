use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(WidgetNodeExt)]
pub fn derive_widget_node_ext(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Enum(data_enum) = input.data else {
        panic!("WidgetNodeExt can only be derived for enums");
    };

    let mut props_arms = Vec::new();
    let mut dyn_id_arms = Vec::new();
    let mut dyn_id_get_arms = Vec::new();
    let mut children_arms = Vec::new();
    let mut widget_type_arms = Vec::new();

    for variant in &data_enum.variants {
        let v_ident = &variant.ident;
        let v_str = v_ident.to_string().to_lowercase();

        match &variant.fields {
            Fields::Named(fields) => {
                let field_idents: Vec<_> =
                    fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();

                let has_props = field_idents.iter().any(|&i| i == "props");
                let has_children = field_idents.iter().any(|&i| i == "children");
                let has_var = field_idents.iter().any(|&i| i == "var");
                let has_node = field_idents.iter().any(|&i| i == "node");

                // Generate type() match arm
                widget_type_arms.push(quote! {
                    Self::#v_ident { .. } => #v_str,
                });

                // Generate props() match arm
                if has_props {
                    props_arms.push(quote! {
                        Self::#v_ident { props, .. } => Some(props),
                    });
                } else {
                    props_arms.push(quote! {
                        Self::#v_ident { .. } => None,
                    });
                }

                // Generate dyn_id()
                if has_props {
                    dyn_id_get_arms.push(quote! {
                        Self::#v_ident { props, .. } => {
                            match props.get("dyn_id") {
                                Some(Property::String(s)) => Some(s.as_str()),
                                _ => None,
                            }
                        }
                    });
                } else {
                    dyn_id_get_arms.push(quote! {
                        Self::#v_ident { .. } => None,
                    });
                }

                // Generate children() match arm
                if has_children {
                    children_arms.push(quote! {
                        Self::#v_ident { children, .. } => children.as_slice(),
                    });
                } else if has_node {
                    children_arms.push(quote! {
                        Self::#v_ident { node, .. } => std::slice::from_ref(node.as_ref()),
                    });
                } else {
                    children_arms.push(quote! {
                        Self::#v_ident { .. } => &[],
                    });
                }

                // Generate setup_dyn_ids() match arm
                if v_ident == "DefWindow" {
                    dyn_id_arms.push(quote! {
                        Self::DefWindow { name, props, node } => Self::DefWindow {
                            name: name.clone(),
                            props: props.clone(),
                            node: Box::new(node.setup_dyn_ids(name)),
                        },
                    });
                } else if v_ident == "Script" {
                    dyn_id_arms.push(quote! {
                        Self::Script { props } => Self::Script {
                            props: with_dyn_id(props.clone(), &format!("{}_script", parent_path)),
                        },
                    });
                } else if has_var && (v_ident == "Poll" || v_ident == "Listen") {
                    let suffix = format!("_{}", v_str);
                    dyn_id_arms.push(quote! {
                        Self::#v_ident { var, props } => Self::#v_ident {
                            var: var.clone(),
                            props: with_dyn_id(props.clone(), &format!("{}{}_{}", parent_path, #suffix, var)),
                        },
                    });
                } else if has_children {
                    dyn_id_arms.push(quote! {
                        Self::#v_ident { props, children } => Self::#v_ident {
                            props: with_dyn_id(props.clone(), parent_path),
                            children: process_children(children, parent_path, #v_str),
                        },
                    });
                } else if has_props {
                    dyn_id_arms.push(quote! {
                        Self::#v_ident { props } => Self::#v_ident {
                            props: with_dyn_id(props.clone(), parent_path),
                        },
                    });
                }
            }
            Fields::Unnamed(fields) => {
                // Generate type() match arm
                widget_type_arms.push(quote! {
                    Self::#v_ident { .. } => #v_str,
                });

                props_arms.push(quote! {
                    Self::#v_ident(..) => None,
                });

                dyn_id_get_arms.push(quote! {
                    Self::#v_ident(..) => None,
                });

                if fields.unnamed.len() == 1 {
                    children_arms.push(quote! {
                        Self::#v_ident(children) => children.as_slice(),
                    });
                } else {
                    children_arms.push(quote! {
                        Self::#v_ident(..) => &[],
                    });
                }

                if v_ident == "Tree" {
                    dyn_id_arms.push(quote! {
                        Self::Tree(children) => Self::Tree(process_children(children, parent_path, "tree")),
                    });
                }
            }
            Fields::Unit => {
                // Generate type() match arm
                widget_type_arms.push(quote! {
                    Self::#v_ident { .. } => #v_str,
                });

                props_arms.push(quote! {
                    Self::#v_ident => None,
                });
            }
        }
    }

    let expanded = quote! {
        impl #name {
            /// Returns the lowercased string representation of the widget type.
            pub fn widget_type(&self) -> &'static str {
                match self {
                    #(#widget_type_arms)*
                }
            }

            /// Returns a reference to the props Map if the variant has one.
            pub fn props(&self) -> Option<&PropertyMap> {
                match self {
                    #(#props_arms)*
                }
            }

            /// Returns the dynamic ID stored in props, if present.
            pub fn dyn_id(&self) -> Option<&str> {
                match self {
                    #(#dyn_id_get_arms)*
                }
            }

            /// Dynamically yields references to child nodes.
            pub fn children(&self) -> &[Self] {
                match self {
                    #(#children_arms)*
                }
            }

            /// Assigns dynamic IDs to nodes and builds paths
            pub fn setup_dyn_ids(&self, parent_path: &str) -> Self {
                fn with_dyn_id(mut props: PropertyMap, dyn_id: &str) -> PropertyMap {
                    props.insert("dyn_id", Property::String(dyn_id.to_string()));
                    props
                }

                fn process_children(
                    children: &[WidgetNode],
                    parent_path: &str,
                    kind: &str,
                ) -> Vec<WidgetNode> {
                    children
                        .iter()
                        .enumerate()
                        .map(|(idx, child)| {
                            let child_path = format!("{}_{}_{}", parent_path, kind, idx);
                            child.setup_dyn_ids(&child_path)
                        })
                        .collect()
                }

                match self {
                    #(#dyn_id_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
