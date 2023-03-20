//! An ORM wrapper for Rust [`leveldb`] [`leveldb::database::kv::KV`] APIs. Use [`bincode`] to encoder / decoder key and object.
//!
//! [`leveldb::database::kv::KV`]: http://skade.github.io/leveldb/leveldb/database/kv/trait.KV.html
//! [`leveldb`]: https://crates.io/crates/leveldb
//!
//! #### Example
//! This example shows the quickest way to get started with feature `macros`
//!
//! ```toml
//! [dependencies]
//! leveldb = "0.8"
//! leveldb-orm = { version = "0.1", features = ["macros"]}
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! ```rust
//! #[cfg(feature = "macros")]
//! mod example {
//!     use leveldb::database::Database;
//!     use leveldb::options::Options;
//!     use leveldb_orm::{KVOrm, KeyOrm, LeveldbOrm};
//!     use serde::{Deserialize, Serialize};
//!
//!     #[derive(LeveldbOrm, Serialize, Deserialize)]
//!     #[leveldb_key(executable, args)]
//!     pub struct Command {
//!         pub executable: u8,
//!         pub args: Vec<String>,
//!         pub current_dir: Option<String>,
//!     }
//!
//!     fn main() {
//!         let cmd = Command {
//!             executable: 1,
//!             args: vec!["arg1".into(), "arg2".into(), "arg3".into()],
//!             current_dir: Some("\\dir".into()),
//!         };
//!
//!         let mut options = Options::new();
//!         options.create_if_missing = true;
//!         let database = Database::open(std::path::Path::new("./mypath"), options).unwrap();
//!
//!         cmd.put(&database).unwrap();
//!
//!         let key = Command::encode_key((&cmd.executable, &cmd.args)).unwrap();
//!         // or `let key = cmd.key().unwrap();`
//!         Command::get(&database, &key).unwrap();
//!
//!         Command::delete(&database, false, &key).unwrap();
//!     }
//! }
//! ```

#[cfg(feature = "macros")]
pub use ::leveldb_orm_derive::LeveldbOrm;

use leveldb::database::batch::Writebatch;
use leveldb::database::Database;
use leveldb::kv::KV;
use leveldb::options::ReadOptions;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

/// The key for leveldb, which impled [`db_key::Key`]. (db-key 0.0.5 only impl it for i32)
/// You can serialize you key to Vec<u8> / &\[u8\] and into EncodedKey.
///
/// [`db_key::Key`]: https://crates.io/crates/db-key/0.0.5
#[derive(Debug, PartialEq)]
pub struct EncodedKey<T: 'static> {
    pub inner: Vec<u8>,
    phantom: PhantomData<T>,
}

impl<T> db_key::Key for EncodedKey<T> {
    #[inline]
    fn from_u8(key: &[u8]) -> Self {
        EncodedKey {
            inner: key.into(),
            phantom: PhantomData,
        }
    }
    #[inline]
    fn as_slice<S, F: Fn(&[u8]) -> S>(&self, f: F) -> S {
        f(&self.inner)
    }
}

impl<T> From<Vec<u8>> for EncodedKey<T> {
    #[inline]
    fn from(inner: Vec<u8>) -> Self {
        EncodedKey {
            inner,
            phantom: PhantomData,
        }
    }
}

impl<T> From<&[u8]> for EncodedKey<T> {
    #[inline]
    fn from(v: &[u8]) -> Self {
        EncodedKey {
            inner: v.into(),
            phantom: PhantomData,
        }
    }
}

/// Interface of key encode / decode
pub trait KeyOrm<'a>: Sized {
    type KeyType: DeserializeOwned;
    type KeyTypeRef: Serialize + 'a;

    /// Without `macros` feature, you can impl `encode_key` by yourself
    #[cfg(not(feature = "macros"))]
    fn encode_key(key: Self::KeyTypeRef) -> Result<EncodedKey<Self>>;

    /// With `macros` feature, the key encodes by [`bincode`]
    #[cfg(feature = "macros")]
    #[inline]
    fn encode_key(key: Self::KeyTypeRef) -> Result<EncodedKey<Self>> {
        bincode::serialize(&key)
            .map(EncodedKey::from)
            .map_err(|e| e.into())
    }

    /// Without `macros` feature, you can impl `decode_key` by yourself
    #[cfg(not(feature = "macros"))]
    fn decode_key(data: &EncodedKey<Self>) -> Result<Self::KeyType>;

    /// With `macros` feature, the key decodes by [`bincode`]
    #[cfg(feature = "macros")]
    #[inline]
    fn decode_key(data: &EncodedKey<Self>) -> Result<Self::KeyType> {
        bincode::deserialize(&data.inner).map_err(|e| e.into())
    }

    /// `#[derive(LeveldbOrm)]` + `#[leveldb_key(...)]` could auto impl this function, without derive macro you can impl it manully
    fn key(&self) -> Result<EncodedKey<Self>>;
}

/// An orm version of [`leveldb::database::kv::KV`](http://skade.github.io/leveldb/leveldb/database/kv/trait.KV.html)
pub trait KVOrm<'a>: KeyOrm<'a> + Serialize + DeserializeOwned {
    /// Encode `Self` by [`bincode`]
    #[inline]
    fn encode(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| e.into())
    }

    /// Decode to `Self` by [`bincode`]
    #[inline]
    fn decode(data: &[u8]) -> Result<Self> {
        bincode::deserialize(data).map_err(|e| e.into())
    }

    /// Refer to [leveldb::database::kv::KV::put](http://skade.github.io/leveldb/leveldb/database/kv/trait.KV.html#tymethod.put)
    fn put_sync(&self, db: &Database<EncodedKey<Self>>, sync: bool) -> Result<()> {
        let key = self.key()?;
        let value = self.encode()?;
        db.put(leveldb::options::WriteOptions { sync }, key, &value)
            .map_err(|e| e.into())
    }

    /// With default sync = false
    fn put(&self, db: &Database<EncodedKey<Self>>) -> Result<()> {
        self.put_sync(db, false)
    }

    /// Refer to [leveldb::database::kv::KV::get](http://skade.github.io/leveldb/leveldb/database/kv/trait.KV.html#tymethod.get)
    fn get_with_option(
        db: &Database<EncodedKey<Self>>,
        options: ReadOptions<'a, EncodedKey<Self>>,
        key: &EncodedKey<Self>,
    ) -> Result<Option<Self>> {
        if let Some(data) = db.get(options, key)? {
            Ok(Some(bincode::deserialize(&data)?))
        } else {
            Ok(None)
        }
    }

    /// With default `ReadOptions`
    fn get(db: &Database<EncodedKey<Self>>, key: &EncodedKey<Self>) -> Result<Option<Self>> {
        Self::get_with_option(db, ReadOptions::new(), key)
    }

    /// Refer to [leveldb::database::kv::KV::delete](http://skade.github.io/leveldb/leveldb/database/kv/trait.KV.html#tymethod.delete)
    fn delete(db: &Database<EncodedKey<Self>>, sync: bool, key: &EncodedKey<Self>) -> Result<()> {
        db.delete(leveldb::options::WriteOptions { sync }, key)
            .map_err(|e| e.into())
    }
}

/// An orm version of [`leveldb::database::batch::Writebatch::put`](http://skade.github.io/leveldb/leveldb/database/batch/struct.Writebatch.html#method.put)
pub trait WritebatchOrm<'a>: KVOrm<'a> {
    fn put_batch(&self, batch: &mut Writebatch<EncodedKey<Self>>) -> Result<()> {
        let key = self.key()?;
        let value = self.encode()?;
        batch.put(key, &value);
        Ok(())
    }
}

impl<'a, T: KeyOrm<'a> + Serialize + DeserializeOwned> KVOrm<'a> for T {}
impl<'a, T: KVOrm<'a>> WritebatchOrm<'a> for T {}
