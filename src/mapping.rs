use log::debug;

use crate::{
    config::{ConfigMap, ControllerInputEvent, ControllerOutputEvent},
    device::{get_device_info, DeviceInfo},
    error::{FatalError, NonFatalError},
    ew_device::Device,
    ew_types::{AbsoluteAxisType, InputEvent, KeyCode, Synchronization},
    output_event::{
        AbsAxisOutputEvent, FilteredAbsAxisOutputEvent, KeyOutputEvent, OutputEvent,
        SyncOutputEvent,
    },
};
use std::{collections::HashMap, io::Error};

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
