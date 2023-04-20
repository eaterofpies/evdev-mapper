use crate::{
    ew_types::{self, AbsoluteAxisType, InputEvent, KeyCode, Synchronization},
    NonFatalError,
};
use evdev::InputEventKind;
use serde::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Display, Formatter},
    fs::File,
    io,
};

#[derive(Debug)]
pub enum FatalError {
    Str(&'static str),
    Io(io::Error),
    SerdeYaml(serde_yaml::Error),
}

impl Display for FatalError {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Io(e) => Display::fmt(e, f),
            Self::SerdeYaml(e) => Display::fmt(e, f),
            Self::Str(e) => Display::fmt(e, f),
        }
    }
}

impl Error for FatalError {}

impl From<io::Error> for FatalError {
    fn from(err: io::Error) -> FatalError {
        FatalError::Io(err)
    }
}

impl From<serde_yaml::Error> for FatalError {
    fn from(err: serde_yaml::Error) -> FatalError {
        FatalError::SerdeYaml(err)
    }
}

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
    input_event: ControllerInputEvent,
    output_event: ControllerOutputEvent,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ControllerInputEvent {
    AbsAxis(AbsoluteAxisType),
    Key(KeyCode),
    Synchronization(Synchronization),
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
            _ => Err(NonFatalError::Str(String::from(
                "Conversion from {:?} to ControllerEvent not implemented",
            ))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct FilteredKeyMapping {
    pub min: i32,
    pub max: i32,
    pub key: KeyCode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(untagged)]
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

fn mappings_to_map(
    mappings: Vec<EventMapping>,
) -> HashMap<ControllerInputEvent, ControllerOutputEvent> {
    let map: HashMap<_, _> = mappings
        .into_iter()
        .map(|m| (m.input_event, m.output_event))
        .collect();
    map
}
