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

    /// Internal flag for if there was an api error
    #[serde(skip_serializing, skip_deserializing)]
    errored: bool,

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
            errored: false,
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
        let win = egui::Window::new("API key error")
            .open(&mut self.errored)
            .collapsible(false)
            .resizable(false);

        win.show(ctx, |ui| {
            // Error modal
            ui.centered_and_justified( |ui| {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    ui.set_height(50.0);
                    ui.label("API key is not valid");

                    ui.collapsing("More information", |ui| {
                        ui.label(format!("API key entered: {}", self.apikey))
                    });
                });
            });
        });

        // Strip for layouting
        StripBuilder::new(container)
            .size(Size::exact(60.0)) // Col 1: UI edittext
            .size(Size::exact(90.0)) // Col 2: Time left in hospital
            .size(Size::remainder()) // Col 3: The username
            .horizontal(|mut strip| {

                let mut input = self.id.to_string();

                // UI edittext
                strip.cell(|ui| {
                    ui.add(egui::text_edit::TextEdit::singleline(&mut input));
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

                    if self.hosp_timestamp.timestamp() == 0 {
                        ui.style_mut().visuals.override_text_color = Some(Color32::from_rgb(255, 0, 0));
                        let lbl = ui.label(format!("ETA: {}", to_hms(time_diff.ceil() as i64)));

                        lbl.on_hover_ui(|ui| {
                            ui.label("\
                                Internal timestamp exactly 0. This is probably because data\
                                hasn't been reloaded, or the id doesn't exist.\
                            ");
                        });
                    }
                    else {
                        let lbl = ui.label(format!("ETA: {}", to_hms(time_diff.ceil() as i64)));
                        lbl.on_hover_ui(|ui| {
                            ui.label("Time to leave hospital");
                        });
                    }
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

        let resp: ApiResponse = executor::block_on(run_request(&built))?;

        let hosp_datetime = DateTime::from_timestamp(
            resp.states["hospital_timestamp"],
            0
        ).unwrap();

        self.hosp_timestamp = hosp_datetime;
        self.name = resp.name;

        Ok(())
    }
}