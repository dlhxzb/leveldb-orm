use leveldb::database::Database;
use leveldb::iterator::Iterable;
use leveldb::options::{Options, ReadOptions};
use leveldb_orm::{KVOrm, KeyOrm, LeveldbOrm};
use serde::{Deserialize, Serialize};
use tempdir::TempDir;

#[test]
fn test_methods() {
    #[derive(Debug, Clone, Serialize, LeveldbOrm, Deserialize, PartialEq)]
    #[leveldb_key(executable, args)]
    struct Command {
        pub executable: u8,
        pub args: Vec<String>,
        pub current_dir: Option<String>,
    }

    let cmd = Command {
        executable: 1,
        args: vec!["arg1".into(), "arg2".into(), "arg3".into()],
        current_dir: Some("\\dir".into()),
    };

    let tempdir = TempDir::new("demo").unwrap();
    let path = tempdir.path();

    let mut options = Options::new();
    options.create_if_missing = true;
    let database = match Database::open(path, options) {
        Ok(db) => db,
        Err(e) => {
            panic!("failed to open database: {:?}", e)
        }
    };

    match cmd.put(&database) {
        Ok(_) => (),
        Err(e) => {
            panic!("failed to write to database: {:?}", e)
        }
    };

    let read_opts = ReadOptions::new();
    let mut iter = database.iter(read_opts);
    let entry = iter
        .next()
        .map(|(k, v)| {
            (
                Command::decode_key(&k).unwrap(),
                Command::decode(&v).unwrap(),
            )
        })
        .unwrap();
    dbg!(&entry);
    assert_eq!(entry, ((cmd.executable, cmd.args.clone()), cmd));
}
