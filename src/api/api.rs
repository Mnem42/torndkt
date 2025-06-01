use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use torn_api::request::{ApiRequest, IntoRequest};
use torn_api::request::models::UserRequest;
use crate::ExampleApp;

/// Player info response
#[derive(Deserialize, Serialize, Debug,  PartialEq, Clone)]
pub struct PlayerInfo {
    pub name: String,

    /// Hospital timestamp, jail timestamp
    pub states: HashMap<String, i64>,
}

/// Type that can be either a number or a string
#[derive(Deserialize, Serialize, Debug, Clone, PartialOrd, PartialEq)]
#[serde(untagged)]
pub enum NumOrString{
    Num(i64),
    String(String)
}

/// Response for an access error
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct AccessErrorStructure{
    pub error: HashMap<String, NumOrString>
}

/// Error returned when attempting to send a tornapi request
#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
pub enum GetInfoError{
    InvalidId,
    WrongKey,
    Other(u8)
}

// Make it usable as an error
impl Display for GetInfoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self{
            GetInfoError::InvalidId => write!(f, "Invalid Id"),
            GetInfoError::WrongKey => write!(f, "Wrong Key"),
            GetInfoError::Other(x) => write!(f, "Other API error: {}", x),
        }
    }
}
impl Error for GetInfoError {}

/// Get the information for a player. Generic, and errors should be checked in
/// calling code.
pub(self) async fn get_player_info<RJT: DeserializeOwned>(req: &ApiRequest, section: &str) -> Result<RJT, ()> {
    let mut start = "https://api.torn.com/v2/".to_string();

    start.push_str(section);
    for i in req.parameters.iter().enumerate(){
        start.push_str(format!("{}{}={}",
                               if i.0 == 0 {'?'} else {'&'},
                               i.1.0,
                               i.1.1).as_str()
        );
    }

    let ret = reqwest::get(&start).await.unwrap();

    println!("{:?}", ret);

    if let Ok(x) = ret.json::<RJT>().await {
        Ok(x)
    }
    else{
        Err(())
    }
}

impl ExampleApp {
    /// Update the hospitalisation timestamp map
    pub async fn update_hosp_time(&mut self, id: u32) -> Result<PlayerInfo, GetInfoError> {
        // Make the request string
        let req = UserRequest::builder()
            .api_key_public(self.apikey.clone())
            .id(id.to_string())
            .build()
            .into_request();

        // Get the normal info
        let resp = get_player_info::<PlayerInfo>(&req.1, "user").await;

        // If that's what was sent, parse and collect
        if let Ok(x) = resp {
            let hosp_datetime = DateTime::from_timestamp(
                x.states["hospital_timestamp"],
                0
            ).unwrap();

            self.hosp_map.insert(x.name.clone(), hosp_datetime);
            Ok(x.clone())
        }
        else{
            // If it's actually an error, figure out what it is and propogate
            if let Ok(x) = get_player_info::<AccessErrorStructure>(&req.1, "user").await{
                match x.error["code"]{
                    NumOrString::Num(code) => {match code{
                        6 => Err(GetInfoError::InvalidId),
                        2 => Err(GetInfoError::WrongKey),
                        x => Err(GetInfoError::Other(x as u8))
                    }}
                    _ => panic!("Should not get here!")
                }
            }
            // If it reaches here, there's something wrong
            else{
                panic!("Should not get here!");
            }
        }
    }
}