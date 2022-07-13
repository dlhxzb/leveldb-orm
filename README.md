This crate used [bincode](https://crates.io/crates/bincode) to encoder / decoder key and object, wrapped [leveldb](https://crates.io/crates/leveldb) [KV](http://skade.github.io/leveldb/leveldb/database/kv/trait.KV.html) APIs.
```rust
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[derive(LevelDBOrm)]
#[level_db_key(executable,args)]
struct Command {
    pub executable: u8,
    pub args: Vec<String>,
    pub current_dir: Option<String>,
}
```
  
# Generate code

```rust
    use leveldb::kv::KV as _KV;
    impl<'a> leveldb_orm_trait::KeyOrm<'a> for Command {
        type KeyType = (u8, Vec<String>);
        type KeyTypeRef = (&'a u8, &'a Vec<String>);
        #[inline]
        fn encode_key(
            key: &Self::KeyTypeRef,
        ) -> std::result::Result<leveldb_orm_trait::EncodedKey<Self>, Box<dyn std::error::Error>>
        {
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
        fn key(
            &self,
        ) -> std::result::Result<leveldb_orm_trait::EncodedKey<Self>, Box<dyn std::error::Error>>
        {
            Self::encode_key(&(&self.executable, &self.args))
        }
    }
    impl<'a> leveldb_orm_trait::KVOrm<'a> for Command {
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
            db.delete(leveldb::options::WriteOptions { sync }, key)
                .map_err(|e| e.into())
        }
    }
```