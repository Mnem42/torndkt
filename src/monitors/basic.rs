#![warn(missing_docs)]
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use eframe::egui;
use eframe::egui::{Align, Layout, Ui};
use egui_extras::{Size, StripBuilder};
use futures::executor;
use serde::{Deserialize, Serialize};
use torn_api::request::IntoRequest;
use torn_api::request::models::{UserRequest};
use crate::api::api::{run_request, GetInfoError};
use crate::util::to_hms;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SimpleHospMonitor{
    pub id: u32,
    hosp_timestamp:  DateTime<Utc>,
    errored: bool,
    apikey: String,
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

        container.with_layout(Layout::left_to_right(Align::TOP), |ui| {
            ui.set_height(18.0);

            StripBuilder::new(ui)
                .size(Size::exact(60.0))
                .size(Size::exact(60.0))
                .size(Size::remainder())
                .horizontal(|mut strip| {
                let mut input = self.id.to_string();

                strip.cell(|ui| {
                    ui.set_width(60.0);
                    ui.add(egui::text_edit::TextEdit::singleline(&mut input)
                        .desired_width(60.0));
                });

                if !input.is_empty() {
                    self.id = input
                        .chars()
                        .filter(|x| x.is_numeric())
                        .collect::<String>()
                        .parse()
                        .unwrap()
                    }
                else{
                    self.id = 0
                };

                let time_diff = (self.hosp_timestamp - Utc::now())
                    .as_seconds_f32()
                    .ceil()
                    .clamp(0.0, f32::MAX);

                strip.cell(|ui| {
                    ui.set_width(60.0);
                    ui.label(format!("ETA: {}", to_hms(time_diff.ceil() as i64)))
                        .on_hover_ui(|ui| {
                            ui.label("Time to leave hospital");
                        });
                });
                strip.cell(|ui| {
                    ui.set_width(60.0);
                    ui.label(self.name.clone());
                });
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