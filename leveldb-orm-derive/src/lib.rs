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
    let keys = parse::parse_leveldb_key(&input.attrs)
        .ok_or_else(|| Error::new(span, "Need attributr: `#[leveldb_key(key1, key2, ...)]`"))?;
    let key_types = parse::parse_key_types(&keys, &fields)?;
    let ident = input.ident;

    let res = quote!(
        impl<'a> leveldb_orm::KeyOrm<'a> for #ident {
            type KeyType = (#(#key_types,)*);
            type KeyTypeRef = (#(&'a #key_types,)*);

            #[inline]
            fn key(&self) -> std::result::Result<leveldb_orm::EncodedKey<Self>, Box<dyn std::error::Error>> {
                Self::encode_key((#(&self.#keys,)*))
            }
        }
    );
    Ok(res.into())
}

mod parse {
    use syn::{
        parse_quote, Attribute, Error, Fields, Ident, Meta, MetaList, NestedMeta, Path, Result,
        Type,
    };

    pub fn parse_leveldb_key(attrs: &[Attribute]) -> Option<Vec<Ident>> {
        attrs.iter().find_map(|attr| {
            attr.parse_meta().ok().and_then(|meta| {
                let path_name: Path = parse_quote!(leveldb_key);
                if meta.path() == &path_name {
                    if let Meta::List(MetaList { nested, .. }) = meta {
                        let keys = nested.iter().filter_map(parse_key).collect::<Vec<_>>();
                        if keys.is_empty() {
                            None
                        } else {
                            Some(keys)
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        })
    }

    // Parse a single key
    fn parse_key(nested_meta: &NestedMeta) -> Option<Ident> {
        if let NestedMeta::Meta(Meta::Path(Path { segments, .. })) = nested_meta {
            segments.first().map(|seg| seg.ident.clone())
        } else {
            None
        }
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
