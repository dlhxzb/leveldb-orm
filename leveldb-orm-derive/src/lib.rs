use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Field, Ident, Result};

#[proc_macro_derive(LevelDBOrm, attributes(level_db_key))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_orm(input).map_or_else(|e| TokenStream::from(e.to_compile_error()), |r| r)
}

fn derive_orm(input: DeriveInput) -> Result<TokenStream> {
    // dbg!(&input);
    let fields = if let Data::Struct(data_struct) = input.data {
        data_struct.fields
    } else {
        return Err(Error::new(input.span(), "LevelDBOrm macro for Struct only"));
    };
    let keys = search::parse_level_db_key(&input.attrs);
    dbg!(&keys);
    todo!()
}

mod search {
    use syn::{parse_quote, Attribute, Ident, Meta, MetaList, NestedMeta, Path};

    pub fn parse_level_db_key(attrs: &[Attribute]) -> Option<Vec<Ident>> {
        attrs.iter().find_map(|attr| {
            attr.parse_meta().ok().and_then(|meta| {
                let path_name: Path = parse_quote!(level_db_key);
                if meta.path() == &path_name {
                    if let Meta::List(MetaList { nested, .. }) = meta {
                        let keys = nested
                            .iter()
                            .filter_map(|nested_meta| parse_key(nested_meta))
                            .collect::<Vec<_>>();
                        Some(keys)
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
}
