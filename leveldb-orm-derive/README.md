Use `LevelDBOrm` + `level_db_key` to auto impl trait in [leveldb-orm](https://crates.io/crates/leveldb-orm)

```rust
#[derive(LevelDBOrm)]
#[level_db_key(executable, args)]
struct Command {
    pub executable: u8,
    pub args: Vec<String>,
    pub current_dir: Option<String>,
}
```
  
# Generate code

```rust
impl<'a> leveldb_orm::KeyOrm<'a> for Command {
    type KeyType = (u8, Vec<String>);
    type KeyTypeRef = (&'a u8, &'a Vec<String>);
    #[inline]
    fn key(
        &self,
    ) -> std::result::Result<leveldb_orm::EncodedKey<Self>, Box<dyn std::error::Error>> {
        Self::encode_key((&self.executable, &self.args))
    }
}
```