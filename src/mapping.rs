use crate::{
    config::{ConfigMap, ControllerEvent},
    ew_device::Device,
    ew_types::{AbsInfo, AbsoluteAxisType, Key, Synchronization},
};
use std::{
    collections::{HashMap, HashSet},
    io::Error,
};

struct DeviceInfo {
    axis_info: HashMap<AbsoluteAxisType, AbsInfo>,
    key_info: HashSet<Key>,
}

fn get_device_info(device: &Device) -> Result<DeviceInfo, Error> {
    let key_info: HashSet<Key> = device.supported_keys();

    let axis_info = device.get_abs_state()?;

    Ok(DeviceInfo {
        axis_info,
        key_info,
    })
}

#[derive(Clone)]
pub struct AbsAxisOutputEvent {
    pub axis_type: AbsoluteAxisType,
    pub axis_info: AbsInfo,
}

#[derive(Clone)]
pub enum OutputEvent {
    AbsAxis(AbsAxisOutputEvent),
    Key(Key),
    Synchronization(Synchronization),
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
    input: &Key,
    output: &ControllerEvent,
    dev_info: &DeviceInfo,
) -> std::result::Result<OutputEvent, &'static str> {
    if dev_info.key_info.contains(input) {
        match output {
            ControllerEvent::AbsAxis(_) => Err("failed to map key event to absaxis"),
            ControllerEvent::Key(k) => Ok(OutputEvent::Key(k.clone())),
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
        ControllerEvent::Synchronization(a) => Ok(OutputEvent::Synchronization(a.clone())),
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
