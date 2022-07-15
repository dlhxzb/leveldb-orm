#[cfg(feature = "macros")]
mod drive_impl {
    use leveldb::batch::Batch;
    use leveldb::database::batch::Writebatch;
    use leveldb::database::Database;
    use leveldb::iterator::Iterable;
    use leveldb::options::{Options, ReadOptions, WriteOptions};
    use leveldb_orm::{KVOrm, KeyOrm, LeveldbOrm, WritebatchOrm};
    use serde::{Deserialize, Serialize};
    use tempdir::TempDir;

    #[derive(Debug, Clone, Serialize, LeveldbOrm, Deserialize, PartialEq)]
    #[leveldb_key(executable, args)]
    struct Command {
        pub executable: u8,
        pub args: Vec<String>,
        pub current_dir: Option<String>,
    }

    #[test]
    fn test_methods() {
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

        // test KVOrm::put
        cmd.put(&database).unwrap();

        let res = Command::get(&database, &key).unwrap();
        assert_eq!(res, Some(cmd.clone()));

        // test KVOrm::delete
        Command::delete(&database, false, &key).unwrap();

        // test KVOrm::get
        assert!(Command::get(&database, &key).unwrap().is_none());

        let mut batch = Writebatch::new();
        cmd.put_batch(&mut batch).unwrap();
        database.write(WriteOptions::new(), &batch).unwrap();

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
        // test WritebatchOrm::put
        assert_eq!(entry, ((cmd.executable, cmd.args.clone()), cmd));
    }
}
