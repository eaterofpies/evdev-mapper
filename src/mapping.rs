use log::debug;

use crate::{
    config::{ConfigMap, ControllerInputEvent, ControllerOutputEvent, FilteredKeyMapping},
    device::{get_device_info, DeviceInfo},
    error::{FatalError, NonFatalError},
    ew_device::Device,
    ew_types::{AbsInfo, AbsoluteAxisType, InputEvent, KeyCode, Synchronization},
};
use std::{collections::HashMap, io::Error};

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

struct DeviceMapping {
    input_to_output_mapping: HashMap<ControllerInputEvent, OutputEvent>,
}

impl DeviceMapping {
    pub fn new(
        io_mapping: &HashMap<ControllerInputEvent, ControllerOutputEvent>,
        device_info: &DeviceInfo,
    ) -> Result<Self, FatalError> {
        let sync_mapping = HashMap::from([(
            ControllerInputEvent::Synchronization(Synchronization(
                evdev::Synchronization::SYN_REPORT,
            )),
            ControllerOutputEvent::Synchronization(Synchronization(
                evdev::Synchronization::SYN_REPORT,
            )),
        )]);

        let all_mapping = sync_mapping.iter().chain(io_mapping.iter());

        let result_or_err: Result<HashMap<_, _>, FatalError> = all_mapping
            .map(|(i, o)| Self::make_output_mapping(i.clone(), o, device_info))
            .collect();
        let result = result_or_err?;

        Ok(DeviceMapping {
            input_to_output_mapping: result,
        })
    }

    pub fn get(&self, input: &ControllerInputEvent) -> Option<&OutputEvent> {
        self.input_to_output_mapping.get(input)
    }

    pub fn list_output_events(&self) -> Vec<&OutputEvent> {
        self.input_to_output_mapping
            .iter()
            .map(|(_i, o)| o)
            .collect()
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
                    Err(FatalError::from("failed to map absaxis event to key"))
                }
                ControllerOutputEvent::Synchronization(_) => Err(FatalError::from(
                    "failed to map absaxis event to synchronization",
                )),
                ControllerOutputEvent::FilteredKeys(f) => Ok(OutputEvent::FilteredAbsAxis(
                    FilteredAbsAxisOutputEvent::new(axis_type.clone(), *axis_info, f.clone()),
                )),
            }
        } else {
            Err(FatalError::from(
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
                    Err(FatalError::from("failed to map key event to absaxis"))
                }
                ControllerOutputEvent::Key(k) => {
                    Ok(OutputEvent::Key(KeyOutputEvent::new(k.clone(), 0)))
                }
                ControllerOutputEvent::Synchronization(_) => Err(FatalError::from(
                    "failed to map key event to synchronization",
                )),
                ControllerOutputEvent::FilteredKeys(_) => {
                    Err(FatalError::from("failed to map key event to filtered keys"))
                }
            }
        } else {
            Err(FatalError::from(
                "Requested input key not present on device",
            ))
        }
    }

    fn make_output_mapping(
        input: ControllerInputEvent,
        output: &ControllerOutputEvent,
        dev_info: &DeviceInfo,
    ) -> Result<(ControllerInputEvent, OutputEvent), FatalError> {
        let output_event = match &input {
            ControllerInputEvent::AbsAxis(a) => Self::map_in_abs_axis(a, output, dev_info)?,
            ControllerInputEvent::Key(k) => Self::map_in_key(k, output, dev_info)?,
            ControllerInputEvent::Synchronization(_a) => {
                OutputEvent::Synchronization(SyncOutputEvent::new())
            }
        };

        Ok((input, output_event))
    }
}

pub struct EventMapping {
    per_device_mappings: HashMap<String, DeviceMapping>,
}

impl EventMapping {
    fn new_dev_mapping(
        path: String,
        device_info: &DeviceInfo,
        mapping: &HashMap<ControllerInputEvent, ControllerOutputEvent>,
    ) -> Result<(String, DeviceMapping), FatalError> {
        let result = DeviceMapping::new(mapping, device_info);
        match result {
            Ok(r) => Ok((path, r)),
            Err(e) => Err(e),
        }
    }

    fn get_device_info(path: String, device: &Device) -> Result<(String, DeviceInfo), Error> {
        Ok((path, get_device_info(device)?))
    }

    pub fn new(
        config: &ConfigMap,
        paths_and_devs: &HashMap<String, Device>,
    ) -> Result<Self, FatalError> {
        let path_and_info_or_error: Result<HashMap<_, _>, Error> = paths_and_devs
            .iter()
            .map(|(p, d)| Self::get_device_info(p.clone(), d))
            .collect();

        let path_and_info = path_and_info_or_error?;

        let mappings_or_error: Result<HashMap<String, DeviceMapping>, FatalError> = config
            .iter()
            .map(|(p, m)| Self::new_dev_mapping(p.clone(), &path_and_info[p], m))
            .collect();

        let mappings = mappings_or_error?;

        Ok(EventMapping {
            per_device_mappings: mappings,
        })
    }

    pub fn get_output_event(
        &self,
        path: &String,
        event: &InputEvent,
    ) -> Result<OutputEvent, NonFatalError> {
        let input_event = ControllerInputEvent::try_from(event)?;

        let output_event = self
            .per_device_mappings
            .get(path)
            .and_then(|m| m.get(&input_event));

        match output_event {
            Some(ev) => Ok(ev.clone_set_value(event.0.value())),
            None => Err(NonFatalError::from(format!(
                "No mapping for event type {:?}",
                event
            ))),
        }
    }

    pub fn list_output_events(&self) -> Vec<&OutputEvent> {
        self.per_device_mappings
            .iter()
            .flat_map(|(_p, m)| m.list_output_events())
            .collect()
    }
}
