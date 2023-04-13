use crate::{
    config::{ConfigMap, ControllerEvent},
    ew_device::Device,
    ew_types::{AbsInfo, AbsoluteAxisType, KeyCode},
};
use std::{
    collections::{HashMap, HashSet},
    io::Error,
};

struct DeviceInfo {
    axis_info: HashMap<AbsoluteAxisType, AbsInfo>,
    key_info: HashSet<KeyCode>,
}

fn get_device_info(device: &Device) -> Result<DeviceInfo, Error> {
    let key_info: HashSet<KeyCode> = device.supported_keys();

    let axis_info = device.get_abs_state()?;

    Ok(DeviceInfo {
        axis_info,
        key_info,
    })
}

#[derive(Clone, Debug)]
pub struct AbsAxisOutputEvent {
    pub axis_type: AbsoluteAxisType,
    pub axis_info: AbsInfo,
}

impl AbsAxisOutputEvent {
    pub fn clone_set_value(&self, value: i32) -> Self {
        AbsAxisOutputEvent {
            axis_type: self.axis_type.clone(),
            axis_info: self.axis_info.clone_set_value(value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyOutputEvent {
    code: KeyCode,
    value: i32,
}

impl KeyOutputEvent {
    pub fn new(code: KeyCode, value: i32) -> Self {
        KeyOutputEvent { code, value }
    }

    pub fn code(&self) -> KeyCode {
        self.code.clone()
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

#[derive(Clone, Debug)]
pub struct SyncOutputEvent {
    code: u16,
    value: i32,
}

impl SyncOutputEvent {
    pub fn new() -> Self {
        Self { code: 0, value: 0 }
    }

    pub fn clone_set_value(&self, value: i32) -> Self {
        Self { code: 0, value }
    }

    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

#[derive(Clone, Debug)]
pub enum OutputEvent {
    AbsAxis(AbsAxisOutputEvent),
    Key(KeyOutputEvent),
    Synchronization(SyncOutputEvent),
}

impl OutputEvent {
    pub fn clone_set_value(&self, value: i32) -> Self {
        match self {
            OutputEvent::AbsAxis(a) => OutputEvent::AbsAxis(a.clone_set_value(value)),
            OutputEvent::Key(k) => OutputEvent::Key(KeyOutputEvent::new(k.code(), value)),
            OutputEvent::Synchronization(s) => {
                OutputEvent::Synchronization(s.clone_set_value(value))
            }
        }
    }
}

fn map_in_abs_axis(
    input: &AbsoluteAxisType,
    output: &ControllerEvent,
    dev_info: &DeviceInfo,
) -> std::result::Result<OutputEvent, &'static str> {
    let this_dev_info = dev_info.axis_info.iter().find(|(k, _v)| *k == input);
    if let Some((_axis_type, axis_info)) = this_dev_info {
        println!("Mapping {:?} to {:?} info {:?}", input, output, axis_info);
        match output {
            ControllerEvent::AbsAxis(a) => Ok(OutputEvent::AbsAxis(AbsAxisOutputEvent {
                axis_type: a.clone(),
                axis_info: *axis_info,
            })),
            ControllerEvent::Key(_) => Err("failed to map absaxis event to key"),
            ControllerEvent::Synchronization(_) => {
                Err("failed to map absaxis event to synchronization")
            }
        }
    } else {
        Err("Requested input axis not present on device")
    }
}

fn map_in_key(
    input: &KeyCode,
    output: &ControllerEvent,
    dev_info: &DeviceInfo,
) -> std::result::Result<OutputEvent, &'static str> {
    if dev_info.key_info.contains(input) {
        match output {
            ControllerEvent::AbsAxis(_) => Err("failed to map key event to absaxis"),
            ControllerEvent::Key(k) => Ok(OutputEvent::Key(KeyOutputEvent::new(k.clone(), 0))),
            ControllerEvent::Synchronization(_) => {
                Err("failed to map key event to synchronization")
            }
        }
    } else {
        Err("Requested input key not present on device")
    }
}

fn make_output_mapping(
    input: &ControllerEvent,
    output: &ControllerEvent,
    dev_info: &DeviceInfo,
) -> Result<OutputEvent, &'static str> {
    match input {
        ControllerEvent::AbsAxis(a) => map_in_abs_axis(a, output, dev_info),
        ControllerEvent::Key(k) => map_in_key(k, output, dev_info),
        ControllerEvent::Synchronization(_a) => {
            Ok(OutputEvent::Synchronization(SyncOutputEvent::new()))
        }
    }
}

fn make_dev_mapping(
    io_mapping: &HashMap<ControllerEvent, ControllerEvent>,
    axis_info: &DeviceInfo,
) -> HashMap<ControllerEvent, OutputEvent> {
    io_mapping
        .iter()
        .map(|(i, o)| (i.clone(), make_output_mapping(i, o, axis_info).unwrap()))
        .collect()
}

pub type EventMapping = HashMap<String, HashMap<ControllerEvent, OutputEvent>>;
pub fn make_mapping(config: &ConfigMap, paths_and_devs: &HashMap<String, Device>) -> EventMapping {
    let path_and_info: HashMap<_, _> = paths_and_devs
        .iter()
        .map(|(p, d)| (p, get_device_info(d).unwrap()))
        .collect();

    config
        .iter()
        .map(|(p, m)| (p.clone(), make_dev_mapping(m, &path_and_info[p])))
        .collect()
}
