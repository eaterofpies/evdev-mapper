use crate::ew_types::{AbsoluteAxisType, KeyCode, Synchronization};
use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::Error};

#[derive(Debug, Deserialize)]
struct Config {
    devices: Vec<DeviceConfig>,
}

#[derive(Debug, Deserialize)]
struct DeviceConfig {
    path: String,
    mappings: Vec<EventMapping>,
}

#[derive(Debug, Deserialize)]
struct EventMapping {
    input_event: ControllerEvent,
    output_event: ControllerEvent,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ControllerEvent {
    AbsAxis(AbsoluteAxisType),
    Key(KeyCode),
    Synchronization(Synchronization),
}

pub type ConfigMap = HashMap<String, HashMap<ControllerEvent, ControllerEvent>>;
pub fn read(path: &String) -> Result<ConfigMap, Error> {
    let file = File::open(path)?;

    let config: Config = serde_yaml::from_reader(file).expect("Could not read values.");

    let config_map: HashMap<_, _> = config
        .devices
        .into_iter()
        .map(|d| (d.path, mappings_to_map(d.mappings)))
        .collect();
    println!("{:?}", config_map);
    Ok(config_map)
}

fn mappings_to_map(mappings: Vec<EventMapping>) -> HashMap<ControllerEvent, ControllerEvent> {
    let map: HashMap<_, _> = mappings
        .into_iter()
        .map(|m| (m.input_event, m.output_event))
        .collect();
    map
}
