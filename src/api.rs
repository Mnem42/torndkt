use std::collections::HashMap;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use torn_api::request::{ApiRequest, IntoRequest};
use torn_api::request::models::UserRequest;
use crate::ExampleApp;

#[derive(Deserialize, Serialize, Debug,  PartialEq, Clone)]
pub struct PlayerInfo {
    pub name: String,

    /// Hospital timestamp, jail timestamp
    pub states: HashMap<String, i64>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialOrd, PartialEq)]
enum NumOrString{
    Num(i64),
    String(String)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccessErrorStructure{
    pub error: HashMap<String, NumOrString>
}

pub enum GetInfoError{
    InvalidId,
    WrongKey,
    Other(u8)
}
pub async fn get_player_info<RJT: DeserializeOwned>(req: &ApiRequest, section: &str) -> Result<RJT, ()> {
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
    pub async fn update_hosp_time(&mut self, id: u32) -> Result<PlayerInfo, GetInfoError> {
        let req = UserRequest::builder()
            .api_key_public(self.apikey.clone())
            .id(id.to_string())
            .build()
            .into_request();

        let resp = get_player_info::<PlayerInfo>(&req.1, "user").await;

        if let Ok(x) = resp {
            let hosp_datetime = DateTime::from_timestamp(
                x.states["hospital_timestamp"],
                0
            ).unwrap();

            self.hosp_map.insert(x.name.clone(), hosp_datetime);
            Ok(x.clone())
        }
        else{
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
            else{
                panic!("Should not get here!");
            }
        }
    }
}