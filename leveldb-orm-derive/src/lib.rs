//! Use `LeveldbOrm` + `leveldb_key` to auto impl trait in [leveldb-orm](https://crates.io/crates/leveldb-orm)
//!
//! ```rust
//! use leveldb_orm::LeveldbOrm;
//!
//! #[derive(LeveldbOrm)]
//! #[leveldb_key(executable, args)]
//! struct Command {
//!     pub executable: u8,
//!     pub args: Vec<String>,
//!     pub current_dir: Option<String>,
//! }
//!  
//! // Generate code
//!
//! // impl<'a> leveldb_orm::KeyOrm<'a> for Command {
//! //     type KeyType = (u8, Vec<String>);
//! //     type KeyTypeRef = (&'a u8, &'a Vec<String>);
//! //     #[inline]
//! //     fn key(
//! //         &self,
//! //     ) -> std::result::Result<leveldb_orm::EncodedKey<Self>, Box<dyn std::error::Error>> {
//! //         Self::encode_key((&self.executable, &self.args))
//! //     }
//! // }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Result};

#[proc_macro_derive(LeveldbOrm, attributes(leveldb_key))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_orm(input).map_or_else(|e| TokenStream::from(e.to_compile_error()), |r| r)
}

fn derive_orm(input: DeriveInput) -> Result<TokenStream> {
    let span = input.span();
    let fields = if let Data::Struct(data_struct) = input.data {
        data_struct.fields
    } else {
        return Err(Error::new(span, "LeveldbOrm macro for Struct only"));
    };

    let mut keys = parse::parse_leveldb_key(&input.attrs)
        .ok_or_else(|| Error::new(span, "Need attributr: `#[leveldb_key(key1, key2, ...)]`"))?;
    let mut key_types = parse::parse_key_types(&keys, &fields)?;

    let (keys, key_types, key_types_ref) = if key_types.len() == 1 {
        let key = keys.pop().unwrap();
        let key_type = key_types.pop().unwrap();
        (
            quote! { &self.#key },
            quote! {#key_type},
            quote! {&'a #key_type},
        )
    } else {
        (
            quote! { (#(&self.#keys,)*) },
            quote! {(#(#key_types,)*)},
            quote! {(#(&'a #key_types,)*)},
        )
    };
    let ident = input.ident;

    let res = quote! {
        impl<'a> leveldb_orm::KeyOrm<'a> for #ident {
            type KeyType = #key_types;
            type KeyTypeRef = #key_types_ref;

            #[inline]
            fn key(&self) -> leveldb_orm::Result<leveldb_orm::EncodedKey<Self>> {
                Self::encode_key(#keys)
            }
        }
    };
    Ok(res.into())
}

mod parse {
    use syn::{Attribute, Error, Fields, Ident, Result, Type};

    pub fn parse_leveldb_key(attrs: &[Attribute]) -> Option<Vec<Ident>> {
        attrs.iter().find_map(|attr| {
            if attr.path().is_ident("leveldb_key") {
                let mut keys = vec![];
                attr.parse_nested_meta(|meta| {
                    if let Some(key) = meta.path.get_ident() {
                        keys.push(key.clone());
                    }
                    Ok(())
                })
                .unwrap();
                (!keys.is_empty()).then_some(keys)
            } else {
                None
            }
        })
    }

    pub fn parse_key_types(keys: &[Ident], fields: &Fields) -> Result<Vec<Type>> {
        let mut res = vec![];
        for key in keys {
            let ty = fields
                .iter()
                .find_map(|field| {
                    field
                        .ident
                        .as_ref()
                        .filter(|ident| ident == &key)
                        .map(|_| field.ty.clone())
                })
                .ok_or_else(|| {
                    Error::new(
                        key.span(),
                        format!("leveldb_key: \"{key}\" not found in struct.",),
                    )
                })?;
            res.push(ty);
        }
        Ok(res)
    }
}
