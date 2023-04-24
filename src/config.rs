use crate::{
    error::{FatalError, NonFatalError},
    ew_types::{self, AbsoluteAxisType, InputEvent, KeyCode, Synchronization},
};
use evdev::InputEventKind;
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
#[serde(untagged)]

pub enum EventMapping {
    KeyEvent {
        input: KeyCode,
        output: KeyCode,
    },
    AbsAxisEvent {
        input: AbsoluteAxisType,
        output: AbsAxisEvent,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum AbsAxisEvent {
    AbsAxis(AbsoluteAxisType),
    FilteredKeys(Vec<FilteredKeyMapping>),
}

impl TryFrom<&InputEvent> for ControllerInputEvent {
    type Error = NonFatalError;

    fn try_from(event: &InputEvent) -> Result<Self, NonFatalError> {
        match event.kind() {
            InputEventKind::Synchronization(s) => Ok(ControllerInputEvent::Synchronization(
                ew_types::Synchronization(s),
            )),
            InputEventKind::Key(k) => Ok(ControllerInputEvent::Key(ew_types::KeyCode(k))),
            InputEventKind::AbsAxis(a) => {
                Ok(ControllerInputEvent::AbsAxis(ew_types::AbsoluteAxisType(a)))
            }
            _ => Err(NonFatalError::from(
                "Conversion from {:?} to ControllerEvent not implemented",
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct FilteredKeyMapping {
    pub min: i32,
    pub max: i32,
    pub key: KeyCode,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ControllerInputEvent {
    AbsAxis(AbsoluteAxisType),
    Key(KeyCode),
    Synchronization(Synchronization),
}

#[derive(Debug)]
pub enum ControllerOutputEvent {
    AbsAxis(AbsoluteAxisType),
    Key(KeyCode),
    Synchronization(Synchronization),
    FilteredKeys(Vec<FilteredKeyMapping>),
}

pub type ConfigMap = HashMap<String, HashMap<ControllerInputEvent, ControllerOutputEvent>>;
pub fn read(path: &String) -> Result<ConfigMap, FatalError> {
    let file = File::open(path)?;

    let config: Config = serde_yaml::from_reader(file)?;

    let config_map: HashMap<_, _> = config
        .devices
        .into_iter()
        .map(|d| (d.path, mappings_to_map(d.mappings)))
        .collect();
    println!("{:?}", config_map);
    Ok(config_map)
}

fn mapping_to_tuple(mapping: EventMapping) -> (ControllerInputEvent, ControllerOutputEvent) {
    match mapping {
        EventMapping::KeyEvent { input, output } => (
            ControllerInputEvent::Key(input),
            ControllerOutputEvent::Key(output),
        ),
        EventMapping::AbsAxisEvent { input, output } => {
            let output = match output {
                AbsAxisEvent::AbsAxis(a) => ControllerOutputEvent::AbsAxis(a),
                AbsAxisEvent::FilteredKeys(k) => ControllerOutputEvent::FilteredKeys(k),
            };
            (ControllerInputEvent::AbsAxis(input), output)
        }
    }
}

fn mappings_to_map(
    mappings: Vec<EventMapping>,
) -> HashMap<ControllerInputEvent, ControllerOutputEvent> {
    let map: HashMap<_, _> = mappings.into_iter().map(mapping_to_tuple).collect();
    map
}
