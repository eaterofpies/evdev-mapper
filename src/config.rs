use crate::event::{AbsoluteAxisType, Key};
use serde::Deserialize;
use std::{collections::HashMap, fs::File};

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
    Key(Key),
}

pub type ConfigMap = HashMap<String, HashMap<ControllerEvent, ControllerEvent>>;
pub fn read() -> ConfigMap {
    let file = File::open("device.conf").unwrap();
    let config: Config = serde_yaml::from_reader(file).expect("Could not read values.");

    let config_map: HashMap<_, _> = config
        .devices
        .into_iter()
        .map(|d| (d.path, mappings_to_map(d.mappings)))
        .collect();
    println!("{:?}", config_map);
    config_map
}

fn mappings_to_map(mappings: Vec<EventMapping>) -> HashMap<ControllerEvent, ControllerEvent> {
    let map: HashMap<_, _> = mappings
        .into_iter()
        .map(|m| (m.input_event, m.output_event))
        .collect();
    map
}
