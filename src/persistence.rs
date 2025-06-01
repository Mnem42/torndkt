use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use crate::ExampleApp;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedData{
    pub tracked_player_list: Vec<u32>,
    pub api_key: String
}

#[derive(Debug)]
pub enum PersistenceError{
    IoError(std::io::Error),
    SerdeError(serde_json::Error)
}

impl From<std::io::Error> for PersistenceError{
    fn from(err: std::io::Error) -> PersistenceError{
        PersistenceError::IoError(err)
    }
}

impl From<serde_json::Error> for PersistenceError{
    fn from(err: serde_json::Error) -> PersistenceError{
        PersistenceError::SerdeError(err)
    }
}

impl Display for PersistenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self{
            PersistenceError::IoError(err) => { write!(f, "File I/O error: {}", err) }
            PersistenceError::SerdeError(err) => { write!(f, "Serde error: {}", err) }
        }
    }
}

impl Error for  PersistenceError{}

impl PersistedData{
    pub fn load(filename: &str) -> Result<PersistedData,PersistenceError>{
        let mut file = std::fs::File::open(filename)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        Ok(serde_json::from_str::<PersistedData>(&*buf)?)
    }

    pub fn save(&self, filename: &str) -> Result<(), PersistenceError>{
        let mut file = std::fs::File::create(filename)?;

        file.write(serde_json::to_string(&*self)?.as_bytes())?;
        Ok(())
    }
}

impl From<ExampleApp> for PersistedData{
    fn from(value: ExampleApp) -> Self {
        Self{
            tracked_player_list: value.ids.to_vec(),
            api_key: value.apikey
        }
    }
}

impl From<&ExampleApp> for PersistedData{
    fn from(value: &ExampleApp) -> Self {
        Self{
            tracked_player_list: value.ids.to_vec().clone(),
            api_key: value.apikey.clone()
        }
    }
}