use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{self, punctuated::Punctuated, token::Comma, Data, DataEnum, DataStruct, Fields, FieldsNamed, FieldsUnnamed, Ident, Variant};

#[proc_macro_derive(LazyDiff)]
pub fn struct_diff_iter_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_struct_diff_iter(&ast)
}

fn impl_data_struct(name: &Ident, data_struct: &DataStruct) -> TokenStream {
    let fields = &data_struct.fields;
    let fields = match fields {
        Fields::Named(FieldsNamed { named, .. }) => named,
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed,
        Fields::Unit => {
            let gen = quote! {
                #[automatically_derived]
                impl LazyDiff for #name {
                    fn lazy_diff_iter<'a, 'b: 'a>(&'a self, other: &'b Self) -> impl Iterator<Item = ::struct_diff_iter::DiffData> {
                        [].into_iter()
                    }
                }
            };
            return gen.into();
        }
    };
    let field_writes = fields.iter().enumerate().map(|(i, f)| {
        let field_name = f
            .ident
            .as_ref()
            .map(|x| x.to_token_stream())
            .unwrap_or(syn::Index::from(i).to_token_stream());
        let field_name_str = field_name.to_string();
        if i == 0 {
            quote! {
                self.#field_name.lazy_diff_iter(&other.#field_name).update(|x| x.field.push(#field_name_str))
            }
        } else {
            quote! {
                .chain(self.#field_name.lazy_diff_iter(&other.#field_name).update(|x| x.field.push(#field_name_str)))
            }
        }
    });

    let gen = quote! {
        #[automatically_derived]
        impl LazyDiff for #name {
            fn lazy_diff_iter<'a, 'b: 'a>(&'a self, other: &'b Self) -> impl Iterator<Item = ::struct_diff_iter::DiffData> {
                use ::struct_diff_iter::itertools::Itertools;
                #(#field_writes)*
            }
        }
    };
    gen.into()
}

fn impl_enum(name: &Ident, variants: &Punctuated<Variant, Comma>) -> TokenStream {
    let variants = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        match &variant.fields {
            Fields::Named(FieldsNamed { named: fields, .. })
            | Fields::Unnamed(FieldsUnnamed {
                unnamed: fields, ..
            }) => {
                let self_field_names = fields.iter().enumerate().map(|(i, f)| {
                    let field_name = f
                        .ident
                        .as_ref()
                        .map(|x| x.to_token_stream())
                        .unwrap_or(syn::Index::from(i).to_token_stream());
                    let field_name_str = field_name.to_string();
                    let self_field_name = format_ident!("self_{}", field_name_str);
                    if f.ident.is_some() {
                        quote! {
                            #field_name: #self_field_name 
                        }
                    } else {
                        quote! {
                            #self_field_name 
                        }
                    }
                });
                let other_field_names = fields.iter().enumerate().map(|(i, f)| {
                    let field_name = f
                        .ident
                        .as_ref()
                        .map(|x| x.to_token_stream())
                        .unwrap_or(syn::Index::from(i).to_token_stream());
                    let field_name_str = field_name.to_string();
                    let other_field_name = format_ident!("other_{}", field_name_str);
                    if f.ident.is_some() {
                        quote! {
                            #field_name: #other_field_name 
                        }
                    } else {
                        quote! {
                            #other_field_name 
                        }
                    }
                });
                let field_writes = fields.iter().enumerate().map(|(i, f)| {
                    let field_name = f
                        .ident
                        .as_ref()
                        .map(|x| x.to_token_stream())
                        .unwrap_or(syn::Index::from(i).to_token_stream());
                    let field_name_str = field_name.to_string();
                    let other_field_name = format_ident!("other_{}", field_name_str);
                    let self_field_name = format_ident!("self_{}", field_name_str);
                    let field_name_str = format!("{}.{}", variant_name.to_string(), field_name.to_string());
                    let iters = if i == 0 {
                        quote! {
                            #self_field_name.lazy_diff_iter(&#other_field_name).update(|x| x.field.push(#field_name_str))
                        }
                    } else {
                        quote! {
                            .chain(#self_field_name.lazy_diff_iter(&#other_field_name).update(|mut x| x.field.push(#field_name_str)))
                        }
                    };
                    iters
                });
                if let Fields::Named(_) = variant.fields {
                    quote! {
                        (#name::#variant_name{#(#self_field_names,)*}, #name::#variant_name{#(#other_field_names,)*}) => {
                            Box::new(#(#field_writes)*)
                        }
                    }
                } else {
                    quote! {
                        (#name::#variant_name(#(#self_field_names,)*), #name::#variant_name(#(#other_field_names,)*)) => {
                            Box::new(#(#field_writes)*)
                        }
                    }

                }
            }
            Fields::Unit => quote! {
                (#name::#variant_name, #name::#variant_name) => Box::new([].into_iter()),
            },
        }
    });
    let gen = quote! {
        #[automatically_derived]
        impl LazyDiff for #name {
            fn lazy_diff_iter<'a, 'b: 'a>(&'a self, other: &'b Self) -> impl Iterator<Item = ::struct_diff_iter::DiffData> {
                use ::struct_diff_iter::itertools::Itertools;
                let iter: Box<dyn Iterator<Item = ::struct_diff_iter::DiffData>> = match (self, other) {
                    #(#variants)*
                    (_, _) => Box::new(
                        [::struct_diff_iter::DiffData {
                            field: ::struct_diff_iter::FieldIdentifier::new(),
                            self_value: self,
                            other_value: other,
                        }]
                        .into_iter(),
                    ),
                };
                iter
            }
        }
    };
    gen.into()
}

fn impl_struct_diff_iter(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    match &ast.data {
        Data::Struct(data_struct) => impl_data_struct(name, data_struct),
        Data::Enum(DataEnum {
            variants,
            ..
        }) => impl_enum(name, variants),
        Data::Union(_) => panic!("Unions are unsupported as they are generally unsafe to compare"),
    }
}
