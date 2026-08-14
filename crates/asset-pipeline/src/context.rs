use std::path::PathBuf;

use crate::task::TaskStore;

pub struct Context {
    pub task_store: TaskStore,
    pub input: PathBuf,
    pub output: PathBuf,
}

impl Context {
    pub fn new(input: PathBuf, output: PathBuf) -> Self {
        Self {
            task_store: TaskStore::new(),
            input,
            output,
        }
    }
}
