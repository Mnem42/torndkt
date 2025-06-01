#![cfg_attr(
    all(
        target_os = "windows",
        not(debug_assertions),
    ),
    windows_subsystem = "windows"
)]

mod api;
mod util;
mod persistence;

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use futures::executor;
use eframe::{egui, Storage};
use eframe::egui::{Button};
use eframe::emath::Vec2;
use egui_extras::{Column, TableBuilder};
use uniquevec::UniqueVec;
use crate::api::api::GetInfoError;
use crate::persistence::PersistedData;
use crate::util::to_hms;

struct ExampleApp {
    hosp_map: HashMap<String, DateTime<Utc>>,
    idselbuf: String,
    ids: UniqueVec<u32>,
    uiscale: f32,
    apikey: String,
    errmodal_open: bool,
    first_update: bool
}

impl Default for ExampleApp{
    fn default() -> Self {
        ExampleApp{
            hosp_map: HashMap::new(),
            idselbuf: String::new(),
            ids: UniqueVec::new(),
            apikey: String::new(),
            uiscale: 1.5,
            errmodal_open: false,
            first_update: true
        }
    }
}

impl ExampleApp {
    fn name() -> &'static str {
        "torndkt v0.1.0"
    }

    fn init(&mut self){
        executor::block_on((async || {
            match self.update_torn().await{
                Ok(_) => (),
                Err(_) => {self.errmodal_open = true;}
            }
        })());
    }

    async fn update_torn(&mut self) -> Result<(),GetInfoError> {
        for i in self.ids.clone() {
            let resp = self.update_hosp_time(i).await?;

            let hosp_datetime = DateTime::from_timestamp(
                    resp.states["hospital_timestamp"],
                    0
                ).unwrap();

            self.hosp_map.insert(resp.name, hosp_datetime);
        }

        Ok(())
    }
}

impl eframe::App for ExampleApp {
    fn save(&mut self, _storage: &mut dyn Storage) {
        let data = PersistedData::from(&*self);
        data.save("persistence.json").unwrap();
    }

    #[tokio::main]
    async fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(self.uiscale);

        if self.first_update {
            self.init();
        }

        self.first_update = false;
        let win = egui::Window::new("API key error")
            .open(&mut self.errmodal_open)
            .collapsible(false)
            .resizable(false);

        win.show(ctx, |ui| {
            ui.centered_and_justified( |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_height(50.0);
                    ui.label("API key is not valid");

                    ui.collapsing("More information", |ui| {
                        ui.label(format!("API key entered: {}", self.apikey))
                    });
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                ui.label("API key:");

                ui.add(egui::TextEdit::singleline(&mut self.apikey));
            });

            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                // Reload button
                if ui.button("Reload").clicked() {
                    executor::block_on((async || {
                        match self.update_torn().await{
                            Ok(_) => (),
                            Err(x) => {match x{
                                GetInfoError::WrongKey => self.errmodal_open = true,
                                GetInfoError::InvalidId => {},
                                GetInfoError::Other(x) =>  println!("Error: {:?}", x),
                            }}
                        }
                    })());

                };

                ui.label("ID to add:");

                ui.add(egui::TextEdit::singleline(&mut self.idselbuf).char_limit(8).desired_width(60.0));

                self.idselbuf = self.idselbuf.chars().filter(|x| x.is_numeric()).collect();

                if ui.add(Button::new("Add user").min_size(Vec2::new(30.0,0.0))).clicked(){
                    println!("{}",self.idselbuf);
                    self.ids.push(self.idselbuf.parse().unwrap());
                }
            });

            let table = TableBuilder::new(ui);

            table
                .column(Column::auto().resizable(true).at_least(90.0))
                .column(Column::remainder())
                .header(15.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("User");
                    });
                    header.col(|ui| {
                        ui.heading("Time to exit hosp");
                    });
                })
                .body(|mut body| {
                    self.hosp_map.iter().for_each( |x|
                        body.row(15.0, |mut row| {
                            row.col(|ui| {
                                ui.label(x.0);
                            });
                            row.col(|ui| {
                                let time_gap = x.1.timestamp() - Utc::now().timestamp();

                                ui.label(to_hms(time_gap.clamp(0, i64::MAX)));
                            });
                        })
                    );
                });

        });
    }
}

fn main() -> eframe::Result<()> {
    let result = PersistedData::load("persistence.json");
    let mut app = ExampleApp::default();

    // If error, do nothing. Otherwise, actually use the data
    if let Ok(x) = result{
        app.ids = x.tracked_player_list.into();
        app.apikey = x.api_key;
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_maximize_button(false)
            .with_max_inner_size(Vec2::new(268.5, 1000.0))
            .with_min_inner_size(Vec2::new(267.5, 0.0))
            .with_inner_size(Vec2::new(268.0,150.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        ExampleApp::name(),
        native_options,
        Box::new(|_| Ok(Box::new(app))),
    )
}