use leveldb_orm::LeveldbOrm;
use serde::{Deserialize, Serialize};

#[derive(LeveldbOrm, Serialize, Deserialize)]
#[leveldb_key(executable, args)]
pub struct Command {
    pub executable: u8,
    pub args: Vec<String>,
    pub current_dir: Option<String>,
}
