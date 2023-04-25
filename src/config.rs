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

#[derive(Clone, Debug, Deserialize)]
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

impl From<AbsoluteAxisType> for AbsAxisEvent {
    fn from(t: AbsoluteAxisType) -> Self {
        AbsAxisEvent::AbsAxis(t)
    }
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

impl From<KeyCode> for ControllerInputEvent {
    fn from(k: KeyCode) -> Self {
        ControllerInputEvent::Key(k)
    }
}

impl From<AbsoluteAxisType> for ControllerInputEvent {
    fn from(a: AbsoluteAxisType) -> Self {
        ControllerInputEvent::AbsAxis(a)
    }
}
impl From<EventMapping> for ControllerInputEvent {
    fn from(mapping: EventMapping) -> Self {
        match mapping {
            EventMapping::KeyEvent { input, output: _ } => ControllerInputEvent::Key(input),
            EventMapping::AbsAxisEvent { input, output: _ } => ControllerInputEvent::AbsAxis(input),
        }
    }
}

pub type ConfigMap = HashMap<(String, ControllerInputEvent), EventMapping>;
pub fn read(path: &String) -> Result<ConfigMap, FatalError> {
    let file = File::open(path)?;

    let config: Config = serde_yaml::from_reader(file)?;

    let config_map: HashMap<_, _> = config
        .devices
        .into_iter()
        .flat_map(|d| mappings_to_map(d.path, d.mappings))
        .collect();

    println!("{:?}", config_map);
    Ok(config_map)
}

fn mappings_to_map(
    path: String,
    mappings: Vec<EventMapping>,
) -> HashMap<(String, ControllerInputEvent), EventMapping> {
    mappings
        .into_iter()
        .map(|m| ((path.clone(), m.clone().into()), m))
        .collect()
}
