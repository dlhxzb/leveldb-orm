use leveldb_orm_derive::LevelDBOrm;
use leveldb_orm_derive::*;
use leveldb_orm_trait::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, LevelDBOrm, Serialize, Deserialize, PartialEq)]
#[level_db_key(executable, args)]
struct Command {
    pub executable: u8,
    pub args: Vec<String>,
    pub current_dir: Option<String>,
}
