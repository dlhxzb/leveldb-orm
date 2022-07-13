use leveldb_orm_derive::LevelDBOrm;
use serde::{Deserialize, Serialize};

#[derive(LevelDBOrm, Serialize, Deserialize)]
#[level_db_key(executable, args)]
pub struct Command {
    pub executable: u8,
    pub args: Vec<String>,
    pub current_dir: Option<String>,
}
