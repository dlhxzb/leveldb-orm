use leveldb::database::Database;
use leveldb::options::ReadOptions;
use serde::Serialize;
use std::marker::PhantomData;

// db-key 0.0.5 only impl Key for i32, wrap for orphan rule
#[derive(Debug, PartialEq)]
struct EncodedKey<T: 'static> {
    pub inner: Vec<u8>,
    phantom: PhantomData<T>,
}

impl<T> db_key::Key for EncodedKey<T> {
    fn from_u8(key: &[u8]) -> Self {
        EncodedKey {
            inner: key.into(),
            phantom: PhantomData,
        }
    }
    fn as_slice<S, F: Fn(&[u8]) -> S>(&self, f: F) -> S {
        f(&self.inner)
    }
}

impl<T> From<Vec<u8>> for EncodedKey<T> {
    fn from(inner: Vec<u8>) -> Self {
        EncodedKey {
            inner,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> From<&[u8]> for EncodedKey<T> {
    fn from(v: &[u8]) -> Self {
        EncodedKey {
            inner: v.into(),
            phantom: PhantomData,
        }
    }
}

trait KeyOrm<'a>: Sized {
    type KeyType;
    type KeyTypeRef: Serialize + 'a;

    fn encode_key(
        key: &Self::KeyTypeRef,
    ) -> std::result::Result<EncodedKey<Self>, Box<dyn std::error::Error>>;
    fn decode_key(
        data: &EncodedKey<Self>,
    ) -> std::result::Result<Self::KeyType, Box<dyn std::error::Error>>;
    fn key(&self) -> std::result::Result<EncodedKey<Self>, Box<dyn std::error::Error>>;
}

trait KVOrm<'a>: KeyOrm<'a> {
    fn encode(&self) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>>;
    fn decode(data: &[u8]) -> std::result::Result<Self, Box<dyn std::error::Error>>;
    fn put_sync(
        &self,
        db: &Database<EncodedKey<Self>>,
        sync: bool,
    ) -> std::result::Result<(), Box<dyn std::error::Error>>;
    fn put(
        &self,
        db: &Database<EncodedKey<Self>>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.put_sync(db, false)
    }
    fn get_with_option(
        db: &Database<EncodedKey<Self>>,
        options: ReadOptions<'a, EncodedKey<Self>>,
        key: &EncodedKey<Self>,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>>;
    fn get(
        db: &Database<EncodedKey<Self>>,
        key: &EncodedKey<Self>,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        Self::get_with_option(db, ReadOptions::new(), key)
    }
    fn delete(
        db: &Database<EncodedKey<Self>>,
        sync: bool,
        key: &EncodedKey<Self>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
