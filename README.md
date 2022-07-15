# leveldb-orm

An ORM wrapper for Rust [leveldb] [KV] APIs. Use [bincode] to encoder / decoder key and object.  

[KV]: http://skade.github.io/leveldb/leveldb/database/kv/trait.KV.html
[leveldb]: https://crates.io/crates/leveldb
[bincode]: https://crates.io/crates/bincode

## Rust version policy
Base on rust [leveldb]:

 `leveldb` is built and tested on stable releases of Rust. This are currently `1.31.0` and `1.43.1`. Nightlies
might not build at any point and failures are allowed. There are no known issues with nightlies, though.


## Prerequisites
`snappy` and `leveldb` need to be installed. On Ubuntu, I recommend:

```sh
sudo apt-get install libleveldb-dev libsnappy-dev
```

## Usage
The struct should impl `Serialize` and `Deserialize`, so we need [`serde`](https://crates.io/crates/serde)

### Derive by `macros`
```toml
[dependencies]
leveldb = "0.8"
leveldb-orm = { version = "0.1", features = ["macros"]}
serde = { version = "1.0", features = ["derive"] }
```

Then, on your main.rs:

## Example
```rust
use leveldb::database::Database;
use leveldb::options::Options;
use leveldb_orm::{KVOrm, KeyOrm, LeveldbOrm};
use serde::{Deserialize, Serialize};

#[derive(LeveldbOrm, Serialize, Deserialize)]
#[leveldb_key(executable, args)]
pub struct Command {
    pub executable: u8,
    pub args: Vec<String>,
    pub current_dir: Option<String>,
}

fn main() {
    let cmd = Command {
        executable: 1,
        args: vec!["arg1".into(), "arg2".into(), "arg3".into()],
        current_dir: Some("\\dir".into()),
    };

    let mut options = Options::new();
    options.create_if_missing = true;
    let database = Database::open(std::path::Path::new("./mypath"), options).unwrap();

    cmd.put(&database).unwrap();

    let key = Command::encode_key((&cmd.executable, &cmd.args)).unwrap();
    // or `let key = cmd.key().unwrap();`
    Command::get(&database, &key).unwrap();

    Command::delete(&database, false, &key).unwrap();
}
```

### Without `macros` feature

Only have to impl `KeyOrm` trait manually.

### Test
* cargo test
* cargo test --features "macros"