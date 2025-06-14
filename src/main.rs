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
pub mod monitors;

use crate::api::api::GetInfoError;
use crate::monitors::basic::SimpleHospMonitor;
use crate::monitors::core::{Monitor};
use crate::monitors::selection::MonitorList;
use crate::persistence::PersistedData;
use chrono::{DateTime, Utc};
use eframe::emath::Vec2;
use eframe::{egui, Storage};
use futures::executor;
use std::collections::HashMap;
use uniquevec::UniqueVec;

struct ExampleApp {
    hosp_map: HashMap<String, DateTime<Utc>>,

    monitors: Vec<MonitorList>,
    idselbuf: String,
    ids: UniqueVec<u32>,
    uiscale: f32,
    pub apikey: String,
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
            first_update: true,
            monitors: vec![],
        }
    }
}

impl ExampleApp {
    fn name() -> &'static str {
        "torndkt v0.1.0"
    }

    fn init(&mut self){
        executor::block_on(async {
            match self.update_torn().await{
                Ok(_) => (),
                Err(_) => {self.errmodal_open = true;}
            }
        });

        println!("{:#?}",self.monitors);
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
        let _ = egui::Window::new("API key error")
            .open(&mut self.errmodal_open)
            .collapsible(false)
            .resizable(false);
        

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

                    for i in &mut self.monitors{
                        i.update_torn(&self.apikey);
                    }
                };

                let mut selected = MonitorList::None;

                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", selected))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut selected, MonitorList::None, " ");
                        ui.selectable_value(&mut selected, MonitorList::Simple(SimpleHospMonitor::default()), "Simple monitor");
                    }
                    );

                if selected !=  MonitorList::None{
                    match selected {
                        MonitorList::None => {}
                        MonitorList::Simple(x) => {self.monitors.push(MonitorList::Simple(x))},
                    }
                }
            });

            ui.separator();

            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.set_height(20.0);
                for i in &mut self.monitors{
                    i.update(ui, ctx);
                }
            });

        });
    }
}

fn main() -> eframe::Result<()> {
    let result = PersistedData::load("persistence.json");
    let mut app = ExampleApp::default();

    // If error, do nothing. Otherwise, actually use the data
    if let Ok(x) = result{
        app.apikey = x.api_key;
        app.monitors = x.monitors;
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