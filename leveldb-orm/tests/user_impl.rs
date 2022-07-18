use leveldb::database::Database;
use leveldb::options::Options;
use leveldb_orm::{EncodedKey, KVOrm, KeyOrm};
use serde::{Deserialize, Serialize};
use tempdir::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Command {
    pub executable: u8,
    pub args: Vec<String>,
    pub current_dir: Option<String>,
}

impl<'a> leveldb_orm::KeyOrm<'a> for Command {
    type KeyType = (u8, Vec<String>);
    type KeyTypeRef = (&'a u8, &'a Vec<String>);
    #[inline]
    fn key(&self) -> leveldb_orm::Result<leveldb_orm::EncodedKey<Self>> {
        Self::encode_key((&self.executable, &self.args))
    }
    fn decode_key(data: &EncodedKey<Self>) -> leveldb_orm::Result<Self::KeyType> {
        bincode::deserialize(&data.inner).map_err(|e| e.into())
    }
    fn encode_key(key: Self::KeyTypeRef) -> leveldb_orm::Result<EncodedKey<Self>> {
        bincode::serialize(&key)
            .map(EncodedKey::from)
            .map_err(|e| e.into())
    }
}

#[test]
fn test_user_impl() {
    let cmd = Command {
        executable: 1,
        args: vec!["arg1".into(), "arg2".into(), "arg3".into()],
        current_dir: Some("\\dir".into()),
    };
    let key = cmd.key().unwrap();

    let tempdir = TempDir::new("demo").unwrap();
    let path = tempdir.path();

    let mut options = Options::new();
    options.create_if_missing = true;
    let database = Database::open(path, options).unwrap();

    cmd.put(&database).unwrap();

    let res = Command::get(&database, &key).unwrap();
    assert_eq!(res, Some(cmd.clone()));

    Command::delete(&database, false, &key).unwrap();

    assert!(Command::get(&database, &key).unwrap().is_none());
}
