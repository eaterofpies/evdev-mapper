use std::collections::{HashMap, HashSet};
use evdev::{Device, AbsInfo};
use crate::{config::{ConfigMap, ControllerEvent,}, event::{AbsoluteAxisType, Key}};

struct DeviceInfo {
    axis_info : HashMap<AbsoluteAxisType, AbsInfo>,
    key_info : HashSet<Key>
}

fn get_device_info(dev: &Device) -> DeviceInfo{
    let mut key_info: HashSet<Key> = HashSet::new();
    if let Some(key_attrs) = dev.supported_keys(){
        key_info = key_attrs.iter().map(Key).collect();
    }

    let mut axis_info: HashMap<AbsoluteAxisType, AbsInfo> = HashMap::new();

    if let Some(axis_list) = dev.supported_absolute_axes(){
        if let Ok(abs_infos) = dev.get_abs_state(){
            let ais = abs_infos.to_vec();
            axis_info = axis_list
                .iter()
                .enumerate()
                .map(|(i, a)| (
                    AbsoluteAxisType(a),
                    AbsInfo::new(
                        ais[i].value,
                        ais[i].minimum,
                        ais[i].maximum,
                        ais[i].fuzz,
                        ais[i].flat,
                        ais[i].resolution
                    )
                ))
                .collect();
        }
    }

    DeviceInfo{
        axis_info,
        key_info,
    }
}

#[derive(Clone)]
pub struct AbsAxisOutputEvent {
    pub axis_type: AbsoluteAxisType,
    pub axis_info: AbsInfo
}

#[derive(Clone)]
pub enum OutputEvent{
    AbsAxis(AbsAxisOutputEvent),
    Key(Key)
}

fn map_in_abs_axis(input: &AbsoluteAxisType, output: &ControllerEvent, dev_info: &DeviceInfo) -> std::result::Result<OutputEvent, &'static str> {
    if dev_info.axis_info.contains_key(input){
        match output {
            ControllerEvent::AbsAxis(a)=> Ok(OutputEvent::AbsAxis(AbsAxisOutputEvent {axis_type: a.clone(), axis_info: dev_info.axis_info[a] })),
            ControllerEvent::Key(_) => Err("failed to map absaxis event to key"),
        }
    } else {
        Err("Requested input axis not present on device")
    }
}

fn map_in_key(input: &Key, output: &ControllerEvent, dev_info: &DeviceInfo) -> std::result::Result<OutputEvent, &'static str> {
    if dev_info.key_info.contains(input){
        match output {
            ControllerEvent::AbsAxis(_)=> Err("failed to map key event to absaxis"),
            ControllerEvent::Key(k) => Ok(OutputEvent::Key(k.clone())),
        }
    } else {
        Err("Requested input key not present on device")
    }

}

fn make_output_mapping(input: &ControllerEvent, output: &ControllerEvent, dev_info: &DeviceInfo) -> Result<OutputEvent, &'static str> {
    match input {
        ControllerEvent::AbsAxis(a) => map_in_abs_axis(a, output, dev_info),
        ControllerEvent::Key(k) => map_in_key(k, output, dev_info),
    }
}

fn make_dev_mapping(io_mapping: &HashMap<ControllerEvent,ControllerEvent>, axis_info: &DeviceInfo) -> HashMap<ControllerEvent,OutputEvent>{
    io_mapping
        .iter()
        .map(|(i,o)|(
            i.clone(),
            make_output_mapping(i, o, axis_info).unwrap()
        ))
        .collect()
}

pub type EventMapping = HashMap<String, HashMap<ControllerEvent, OutputEvent>>;
pub fn make_mapping(config: &ConfigMap, paths_and_devs: &HashMap<String, Device>) -> EventMapping {
    let path_and_info: HashMap<_,_> = paths_and_devs
        .iter()
        .map(|(p, d)| (p, get_device_info(d)))
        .collect();

    config
        .iter()
        .map(|(p,m)| (p.clone(), make_dev_mapping(m, &path_and_info[p])))
        .collect()
}
