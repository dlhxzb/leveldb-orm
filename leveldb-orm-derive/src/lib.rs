use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Result};

#[proc_macro_derive(LevelDBOrm, attributes(level_db_key))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_orm(input).map_or_else(|e| TokenStream::from(e.to_compile_error()), |r| r)
}

fn derive_orm(input: DeriveInput) -> Result<TokenStream> {
    // dbg!(&input);
    let span = input.span();
    let fields = if let Data::Struct(data_struct) = input.data {
        data_struct.fields
    } else {
        return Err(Error::new(span, "LevelDBOrm macro for Struct only"));
    };
    let keys = parse::parse_level_db_key(&input.attrs)
        .ok_or_else(|| Error::new(span, "Need attributr: `#[level_db_key(key1, key2, ...)]`"))?;
    let key_types = parse::parse_key_types(&keys, &fields)?;
    let ident = input.ident;

    let res = quote!(const _:() = {
        use leveldb::kv::KV as _KV;
        
        impl<'a> leveldb_orm_trait::KeyOrm<'a> for #ident {
            type KeyType = (#(#key_types,)*);
            type KeyTypeRef = (#(&'a #key_types,)*);

            #[inline]
            fn encode_key(
                key: &Self::KeyTypeRef,
            ) -> std::result::Result<leveldb_orm_trait::EncodedKey<Self>, Box<dyn std::error::Error>> {
                bincode::serialize(key)
                    .map(leveldb_orm_trait::EncodedKey::from)
                    .map_err(|e| e.into())
            }

            #[inline]
            fn decode_key(
                data: &leveldb_orm_trait::EncodedKey<Self>,
            ) -> std::result::Result<Self::KeyType, Box<dyn std::error::Error>> {
                bincode::deserialize(&data.inner).map_err(|e| e.into())
            }

            #[inline]
            fn key(&self) -> std::result::Result<leveldb_orm_trait::EncodedKey<Self>, Box<dyn std::error::Error>> {
                Self::encode_key(&(#(&self.#keys,)*))
            }
        }

        impl<'a> leveldb_orm_trait::KVOrm<'a> for #ident {
            #[inline]
            fn encode(&self) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
                bincode::serialize(self).map_err(|e| e.into())
            }

            #[inline]
            fn decode(data: &[u8]) -> std::result::Result<Self, Box<dyn std::error::Error>> {
                bincode::deserialize(data).map_err(|e| e.into())
            }

            fn put_sync(
                &self,
                db: &leveldb::database::Database<leveldb_orm_trait::EncodedKey<Self>>,
                sync: bool,
            ) -> std::result::Result<(), Box<dyn std::error::Error>> {
                use leveldb_orm_trait::KeyOrm as _KeyOrm;
                
                let key = self.key()?;
                let value = self.encode()?;
                db.put(leveldb::options::WriteOptions { sync }, key, &value)
                    .map_err(|e| e.into())
            }

            fn get_with_option(
                db: &leveldb::database::Database<leveldb_orm_trait::EncodedKey<Self>>,
                options: leveldb::options::ReadOptions<'a, leveldb_orm_trait::EncodedKey<Self>>,
                key: &leveldb_orm_trait::EncodedKey<Self>,
            ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
                if let Some(data) = db.get(options, key)? {
                    Ok(Some(bincode::deserialize(&data)?))
                } else {
                    Ok(None)
                }
            }

            fn delete(
                db: &leveldb::database::Database<leveldb_orm_trait::EncodedKey<Self>>,
                sync: bool,
                key: &leveldb_orm_trait::EncodedKey<Self>,
            ) -> Result<(), Box<dyn std::error::Error>> {
                db.delete(leveldb::options::WriteOptions { sync }, key).map_err(|e| e.into())
            }
        }
    };);
    Ok(res.into())
}

mod parse {
    use syn::{
        parse_quote, Attribute, Error, Fields, Ident, Meta, MetaList, NestedMeta, Path, Result,
        Type,
    };

    pub fn parse_level_db_key(attrs: &[Attribute]) -> Option<Vec<Ident>> {
        attrs.iter().find_map(|attr| {
            attr.parse_meta().ok().and_then(|meta| {
                // dbg!(&meta);
                let path_name: Path = parse_quote!(level_db_key);
                if meta.path() == &path_name {
                    if let Meta::List(MetaList { nested, .. }) = meta {
                        let keys = nested
                            .iter()
                            .filter_map(|nested_meta| parse_key(nested_meta))
                            .collect::<Vec<_>>();
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
                .ok_or(Error::new(
                    key.span(),
                    format!("level_db_key: \"{key}\" not found in struct.",),
                ))?;
            res.push(ty);
        }
        Ok(res)
    }
}
