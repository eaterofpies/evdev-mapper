use log::debug;

use crate::{
    config::{
        ConfigMap, ControllerInputEvent, ControllerOutputEvent, FatalError, FilteredKeyMapping,
    },
    ew_device::Device,
    ew_types::{AbsInfo, AbsoluteAxisType, InputEvent, KeyCode, Synchronization},
};
use std::{
    collections::{HashMap, HashSet},
    io::Error,
};

struct DeviceInfo {
    axis_info: HashMap<AbsoluteAxisType, AbsInfo>,
    key_info: HashSet<KeyCode>,
}

fn get_device_info(path: String, device: &Device) -> Result<(String, DeviceInfo), Error> {
    let key_info: HashSet<KeyCode> = device.supported_keys();

    let axis_info = device.get_abs_state()?;

    Ok((
        path,
        DeviceInfo {
            axis_info,
            key_info,
        },
    ))
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

    pub fn to_evdev_event(&self) -> InputEvent {
        InputEvent::new(
            evdev::EventType::ABSOLUTE,
            self.axis_type.0 .0,
            self.axis_info.0.value(),
        )
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

    pub fn to_evdev_event(&self) -> InputEvent {
        InputEvent::new(evdev::EventType::KEY, self.code().0 .0, self.value())
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

    pub fn to_evdev_event(&self) -> InputEvent {
        InputEvent::new(evdev::EventType::SYNCHRONIZATION, self.code(), self.value())
    }
}

#[derive(Clone, Debug)]
pub struct FilteredAbsAxisOutputEvent {
    axis_type: AbsoluteAxisType,
    axis_info: AbsInfo,
    mappings: Vec<FilteredKeyMapping>,
}

impl FilteredAbsAxisOutputEvent {
    pub fn new(
        input_axis_type: AbsoluteAxisType,
        info: AbsInfo,
        mappings: Vec<FilteredKeyMapping>,
    ) -> Self {
        FilteredAbsAxisOutputEvent {
            axis_type: input_axis_type,
            axis_info: info,
            mappings,
        }
    }
    pub fn codes(&self) -> Vec<KeyCode> {
        self.mappings.iter().map(|f| f.key.clone()).collect()
    }

    pub fn clone_set_value(&self, value: i32) -> Self {
        FilteredAbsAxisOutputEvent {
            axis_type: self.axis_type.clone(),
            axis_info: self.axis_info.clone_set_value(value),
            mappings: self.mappings.clone(),
        }
    }

    fn mapping_to_evdev_event(&self, mapping: &FilteredKeyMapping) -> InputEvent {
        let axis_value = self.axis_info.0.value();
        let mut out_value = 0;
        if axis_value >= mapping.min && axis_value <= mapping.max {
            out_value = 1
        }

        InputEvent::new(evdev::EventType::KEY, mapping.key.0 .0, out_value)
    }

    pub fn to_evdev_events(&self) -> Vec<InputEvent> {
        self.mappings
            .iter()
            .map(|e| self.mapping_to_evdev_event(e))
            .collect()
    }
}
// Can't just use config directly as we need to clone the input axis info and values
#[derive(Clone, Debug)]
pub enum OutputEvent {
    AbsAxis(AbsAxisOutputEvent),
    Key(KeyOutputEvent),
    Synchronization(SyncOutputEvent),
    FilteredAbsAxis(FilteredAbsAxisOutputEvent),
}

impl OutputEvent {
    pub fn clone_set_value(&self, value: i32) -> Self {
        match self {
            OutputEvent::AbsAxis(a) => OutputEvent::AbsAxis(a.clone_set_value(value)),
            OutputEvent::Key(k) => OutputEvent::Key(KeyOutputEvent::new(k.code(), value)),
            OutputEvent::Synchronization(s) => {
                OutputEvent::Synchronization(s.clone_set_value(value))
            }
            OutputEvent::FilteredAbsAxis(f) => {
                OutputEvent::FilteredAbsAxis(f.clone_set_value(value))
            }
        }
    }

    pub fn to_evdev_events(&self) -> Vec<InputEvent> {
        match self {
            OutputEvent::AbsAxis(a) => vec![a.to_evdev_event()],
            OutputEvent::Key(k) => vec![k.to_evdev_event()],
            OutputEvent::Synchronization(s) => vec![s.to_evdev_event()],
            OutputEvent::FilteredAbsAxis(f) => f.to_evdev_events(),
        }
    }
}

fn map_in_abs_axis(
    input: &AbsoluteAxisType,
    output: &ControllerOutputEvent,
    dev_info: &DeviceInfo,
) -> std::result::Result<OutputEvent, FatalError> {
    let this_dev_info = dev_info.axis_info.iter().find(|(k, _v)| *k == input);
    if let Some((axis_type, axis_info)) = this_dev_info {
        debug!("Mapping {:?} to {:?} info {:?}", input, output, axis_info);
        match output {
            ControllerOutputEvent::AbsAxis(a) => Ok(OutputEvent::AbsAxis(AbsAxisOutputEvent {
                axis_type: a.clone(),
                axis_info: *axis_info,
            })),
            ControllerOutputEvent::Key(_) => {
                Err(FatalError::Str("failed to map absaxis event to key"))
            }
            ControllerOutputEvent::Synchronization(_) => Err(FatalError::Str(
                "failed to map absaxis event to synchronization",
            )),
            ControllerOutputEvent::FilteredKeys(f) => Ok(OutputEvent::FilteredAbsAxis(
                FilteredAbsAxisOutputEvent::new(axis_type.clone(), *axis_info, f.clone()),
            )),
        }
    } else {
        Err(FatalError::Str(
            "Requested input axis not present on device",
        ))
    }
}

fn map_in_key(
    input: &KeyCode,
    output: &ControllerOutputEvent,
    dev_info: &DeviceInfo,
) -> std::result::Result<OutputEvent, FatalError> {
    if dev_info.key_info.contains(input) {
        match output {
            ControllerOutputEvent::AbsAxis(_) => {
                Err(FatalError::Str("failed to map key event to absaxis"))
            }
            ControllerOutputEvent::Key(k) => {
                Ok(OutputEvent::Key(KeyOutputEvent::new(k.clone(), 0)))
            }
            ControllerOutputEvent::Synchronization(_) => Err(FatalError::Str(
                "failed to map key event to synchronization",
            )),
            ControllerOutputEvent::FilteredKeys(_) => {
                Err(FatalError::Str("failed to map key event to filtered keys"))
            }
        }
    } else {
        Err(FatalError::Str("Requested input key not present on device"))
    }
}

fn make_output_mapping(
    input: ControllerInputEvent,
    output: &ControllerOutputEvent,
    dev_info: &DeviceInfo,
) -> Result<(ControllerInputEvent, OutputEvent), FatalError> {
    let output_event = match &input {
        ControllerInputEvent::AbsAxis(a) => map_in_abs_axis(a, output, dev_info)?,
        ControllerInputEvent::Key(k) => map_in_key(k, output, dev_info)?,
        ControllerInputEvent::Synchronization(_a) => {
            OutputEvent::Synchronization(SyncOutputEvent::new())
        }
    };

    Ok((input, output_event))
}

fn make_dev_mapping(
    path: String,
    io_mapping: &HashMap<ControllerInputEvent, ControllerOutputEvent>,
    axis_info: &DeviceInfo,
) -> Result<(String, HashMap<ControllerInputEvent, OutputEvent>), FatalError> {
    let sync_mapping = HashMap::from([(
        ControllerInputEvent::Synchronization(Synchronization(evdev::Synchronization::SYN_REPORT)),
        ControllerOutputEvent::Synchronization(Synchronization(evdev::Synchronization::SYN_REPORT)),
    )]);

    let all_mapping = sync_mapping.iter().chain(io_mapping.iter());

    let result: Result<HashMap<_, _>, FatalError> = all_mapping
        .map(|(i, o)| make_output_mapping(i.clone(), o, axis_info))
        .collect();

    match result {
        Ok(r) => Ok((path, r)),
        Err(e) => Err(e),
    }
}

pub type EventMapping = HashMap<String, HashMap<ControllerInputEvent, OutputEvent>>;
pub fn make_mapping(
    config: &ConfigMap,
    paths_and_devs: &HashMap<String, Device>,
) -> Result<EventMapping, FatalError> {
    let path_and_info_or_error: Result<HashMap<_, _>, Error> = paths_and_devs
        .iter()
        .map(|(p, d)| get_device_info(p.clone(), d))
        .collect();

    let path_and_info = path_and_info_or_error?;

    config
        .iter()
        .map(|(p, m)| make_dev_mapping(p.clone(), m, &path_and_info[p]))
        .collect()
}
