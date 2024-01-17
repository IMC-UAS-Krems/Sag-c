use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident, Type};

#[proc_macro_derive(Block, attributes(treat_as, is_optional))]
/// Implements `get_field` method
pub fn all_block_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as syn::DeriveInput);

    // Ensure the derive is only applied to structs
    if let Data::Struct(DataStruct { fields, .. }) = &input.data {
        let struct_name = &input.ident;

        let (field_names, field_types): (Vec<_>, Vec<_>) = match fields {
            Fields::Named(named_fields) => named_fields
                .named
                .iter()
                .map(|f| {
                    let ident = &f
                        .ident
                        .as_ref()
                        .expect("Named fields should have identifiers");
                    let ty = &f
                        .attrs
                        .iter()
                        .find_map(|f| {
                            if f.path().is_ident("treat_as") {
                                let ty = f.parse_args::<Type>().unwrap();
                                Some(quote! { #ty })
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| f.ty.to_token_stream());
                    (ident.to_owned().clone(), ty.clone())
                })
                .unzip(),
            _ => panic!("AllBlock derive can only be applied to structs with named fields"),
        };

        let doc = format!(
            "Returns fields: {}",
            field_names
                .iter()
                .map(|f: &Ident| f.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let gen = quote! {
            impl<'a> #struct_name<'a> {
                #[doc = #doc]
                fn get_fields_from_blocks(
                    blocks: &Blocks<'a>,
                    block_name: &'a str,
                ) -> Result<(#( #field_types, )*), SagError> {
                    let block = blocks
                        .get(block_name)
                        .ok_or_else(|| SagError::missing_section(block_name))?;

                    Ok((
                        #(
                            parse! {block,
                            #field_types,
                            block_name,
                            stringify!(#field_names).replace("r#", "")
                                .as_str()

                            },
                        )*
                    ))
                }
                #[doc = #doc]
                fn get_fields_from_block(
                    block: &Value<'a>,
                    block_name: &'a str,
                ) -> Result<(#( #field_types, )*), SagError> {

                    Ok((
                        #(
                            parse! {block,
                            #field_types,
                            block_name,
                            stringify!(#field_names).replace("r#", "")
                                .as_str()

                            },
                        )*
                    ))
                }
            }
        };

        gen.into()
    } else {
        panic!("AllBlock derive can only be applied to structs");
    }
}
