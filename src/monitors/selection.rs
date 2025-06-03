use std::fmt::Display;
use eframe::egui::{Context, Ui};
use serde::{Deserialize, Serialize};
use crate::api::api::GetInfoError;
use crate::monitors::basic::SimpleHospMonitor;
use crate::monitors::core::Monitor;


/// Enum to encode all monitor types
#[derive(Default, Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub enum MonitorList {
    /// Default, no monitor
    #[default]
    None,

    /// Simple monitor with just hospitalisation time and name
    Simple(SimpleHospMonitor),
}

impl Display for MonitorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MonitorList::Simple(_) => "Simple".to_string(),
            MonitorList::None => "".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Monitor for MonitorList{
    fn update(&mut self, container: &mut Ui, ctx: &Context) {
        match self{
            MonitorList::Simple(x) => {x.update(container, ctx);},
            MonitorList::None => {}
        }
    }

    fn update_torn(&mut self, apikey: &str) -> Result<(), GetInfoError> {
        match self{
            MonitorList::Simple(x) => x.update_torn(apikey),
            MonitorList::None => Ok(())
        }
    }
}