use eframe::egui;
use eframe::egui::Ui;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::api::api::GetInfoError;

/// The monitor trait. All monitors should implement this, but there's not really
/// anything to enforce it.
pub trait Monitor: Serialize + DeserializeOwned{
    /// Run on each egui update
    fn update(&mut self, container: &mut Ui, ctx: &egui::Context);

    /// Update tornapi data
    fn update_torn(&mut self, apikey: &str) -> Result<(), GetInfoError>;
}