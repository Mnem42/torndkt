#![warn(missing_docs)]
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use eframe::egui;
use eframe::egui::{Align, Color32, Label, Layout, Ui};
use egui_extras::{Size, StripBuilder};
use futures::executor;
use serde::{Deserialize, Serialize};
use torn_api::request::IntoRequest;
use torn_api::request::models::{UserRequest};
use crate::api::api::{run_request, GetInfoError};
use crate::monitors::core::Monitor;
use crate::util::to_hms;

/// A simple hospitalisation monitor, that just shows how long it will last and
/// the name of the user, when given an id.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SimpleHospMonitor{
    /// User ID to track
    pub id: u32,

    /// Hospital timestamp, as given by tornapi
    #[serde(skip_serializing, skip_deserializing)]
    hosp_timestamp:  DateTime<Utc>,

    /// Internal flag for api errors
    #[serde(skip_serializing, skip_deserializing)]
    id_error: bool,

    /// Torn API key. Stored internally for error messages
    #[serde(skip_serializing, skip_deserializing)]
    apikey: String,

    /// Internally stored username, as given by the torn api
    #[serde(skip_serializing, skip_deserializing)]
    name: String
}

impl Default for SimpleHospMonitor{
    fn default() -> SimpleHospMonitor{
        SimpleHospMonitor{
            id: 0,
            hosp_timestamp:  Utc::now(),
            id_error: false,
            apikey: String::new(),
            name: String::new()
        }
    }
}

#[derive(Deserialize, Serialize, Debug,  PartialEq, Clone)]
struct ApiResponse {
    pub name: String,

    /// Hospital timestamp, jail timestamp
    pub states: HashMap<String, i64>,
}

impl Monitor for SimpleHospMonitor{
    fn update(&mut self, container: &mut Ui, ctx: &egui::Context) {

        // Strip for layouting
        StripBuilder::new(container)
            .size(Size::exact(60.0)) // Col 1: UI edittext
            .size(Size::exact(90.0)) // Col 2: Time left in hospital
            .size(Size::remainder()) // Col 3: The username
            .horizontal(|mut strip| {

                let mut input = self.id.to_string();

                // UI edittext
                strip.cell(|ui| {
                    if self.id_error{
                        ui.style_mut().visuals.extreme_bg_color =  Color32::from_rgb(255, 0, 0);
                    }

                    let textedit = ui.text_edit_singleline(&mut input);

                    textedit.on_hover_text(if self.id_error {"ID doesn't exist"} else {"ID to query"});
                });

                input = input.chars()
                    .filter(|x| x.is_numeric())
                    .collect::<String>();

                // Safely parse into internally stored id
                if !input.is_empty() {
                    self.id = input
                        .parse()
                        .unwrap()
                }
                else{
                    self.id = 0
                };

                // Get time left in hospital
                let time_diff = (self.hosp_timestamp - Utc::now())
                    .as_seconds_f32()
                    .ceil()
                    .clamp(0.0, f32::MAX);

                // Col 2: Time left in hospital
                strip.cell(|ui| {
                    let lbl = ui.label(format!("ETA: {}", to_hms(time_diff.ceil() as i64)));
                    lbl.on_hover_text("Time to leave hospital");
                });

                // Col 3: Username
                strip.cell(|ui| {
                    ui.label(self.name.clone());
                });
            });
    }

    fn update_torn(&mut self, apikey: &str) -> Result<(),GetInfoError>{
        self.apikey = apikey.to_string();

        let built = UserRequest::builder()
            .id(self.id.to_string())
            .api_key_public(apikey)
            .build()
            .into_request().1;

        let resp: Result<ApiResponse, _> = executor::block_on(run_request(&built));

        match resp {
            Ok(resp) => {
                let hosp_datetime = DateTime::from_timestamp(
                    resp.states["hospital_timestamp"],
                    0
                ).unwrap();

                self.hosp_timestamp = hosp_datetime;
                self.name = resp.name;

                Ok(())
            }
            Err(x) => {
                match x {
                    GetInfoError::InvalidId => {self.id_error = true;}
                    GetInfoError::WrongKey => {}
                    GetInfoError::Other(_) => {}
                }

                Err(x)
            }
        }
    }
}