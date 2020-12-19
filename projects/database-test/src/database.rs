use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Serialize};
use std::cell::RefCell;
use std::cell::RefMut;
use std::fmt::Debug;
use std::rc::Rc;

use crate::consoleLog;

pub enum DatabaseError {
    Uninitialized,
}

#[repr(C)]
pub enum DatabaseResponse {
    JSON(String),
    Snapshot(Vec<u8>),
    Uninitialized,
    SerializationError,
    UnknownQueryError,
    UnknownSaveError,
}

pub struct Database<T> {
    root: Option<Rc<RefCell<T>>>,
}

impl<D> Database<D> {
    pub const fn new() -> Database<D> {
        Database { root: None }
    }
}

impl<T> Database<T>
where
    T: Serialize + DeserializeOwned + Clone + Send + Debug,
{
    pub fn exec_with_db<U>(&mut self, task: U) -> Result<(), DatabaseError>
    where
        U: FnOnce(RefMut<T>),
    {
        match &self.root {
            Some(root) => {
                task(root.borrow_mut());
                Ok(())
            }
            None => Err(DatabaseError::Uninitialized),
        }
    }

    pub fn initialize<U>(&mut self, snapshot: &Vec<u8>, fallback: U)
    where
        U: FnOnce() -> T,
    {
        let root = match bincode::deserialize::<T>(&*snapshot) {
            Ok(root) => root,
            _ => fallback(),
        };
        if self.root.is_none() {
            consoleLog!(serde_json::to_string(&root));
            self.root = Some(Rc::new(RefCell::new(root)));
        }
    }

    pub fn fullJSON(&self) -> DatabaseResponse {
        match &self.root {
            Some(root) => {
                consoleLog!(root);
                match serde_json::to_string(root.as_ref()) {
                    Ok(snapshot) => DatabaseResponse::JSON(snapshot),
                    Err(_) => DatabaseResponse::SerializationError,
                }
            }
            None => DatabaseResponse::Uninitialized,
        }
    }

    pub fn persist(&self) -> DatabaseResponse {
        match &self.root {
            Some(root) => match bincode::serialize(root.as_ref()) {
                Ok(snapshot) => DatabaseResponse::Snapshot(snapshot),
                Err(_) => DatabaseResponse::SerializationError,
            },
            None => DatabaseResponse::Uninitialized,
        }
    }
}
