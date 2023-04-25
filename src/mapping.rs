use crate::{
    config::{self, ConfigMap, ControllerInputEvent},
    device::{get_device_info, DeviceInfo},
    error::{FatalError, NonFatalError},
    ew_device::Device,
    ew_types::{AbsoluteAxisType, InputEvent, Synchronization},
    output_event::{
        AbsAxisOutputEvent, FilteredAbsAxisOutputEvent, KeyOutputEvent, OutputEvent,
        SyncOutputEvent,
    },
};
use std::{collections::HashMap, io::Error};

pub struct EventMapping {
    mappings: HashMap<(String, ControllerInputEvent), OutputEvent>,
}

impl EventMapping {
    fn get_device_info(path: String, device: &Device) -> Result<(String, DeviceInfo), Error> {
        Ok((path, get_device_info(device)?))
    }

    fn make_abs_axis_mapping(
        device_info: &DeviceInfo,
        axis_type: AbsoluteAxisType,
        axis_event: config::AbsAxisEvent,
    ) -> OutputEvent {
        let (_, axis_info) = device_info
            .axis_info
            .iter()
            .find(|(k, _v)| *k == &axis_type)
            .unwrap();
        match axis_event {
            config::AbsAxisEvent::AbsAxis(a) => OutputEvent::AbsAxis(AbsAxisOutputEvent {
                axis_type: a,
                axis_info: *axis_info,
            }),
            config::AbsAxisEvent::FilteredKeys(f) => OutputEvent::FilteredAbsAxis(
                FilteredAbsAxisOutputEvent::new(axis_type.clone(), *axis_info, f),
            ),
        }
    }

    fn make_mapping(
        path: String,
        event: ControllerInputEvent,
        mapping: config::EventMapping,
        device_info: &DeviceInfo,
    ) -> ((String, ControllerInputEvent), OutputEvent) {
        let output = match mapping {
            config::EventMapping::KeyEvent { input: _, output } => {
                OutputEvent::Key(KeyOutputEvent::new(output, 0))
            }
            config::EventMapping::AbsAxisEvent { input, output } => {
                Self::make_abs_axis_mapping(device_info, input, output)
            }
        };
        ((path, event), output)
    }

    fn make_sync_mapping(path: String) -> ((String, ControllerInputEvent), OutputEvent) {
        let input = ControllerInputEvent::Synchronization(Synchronization(
            evdev::Synchronization::SYN_REPORT,
        ));
        let output = OutputEvent::Synchronization(SyncOutputEvent::new());
        ((path, input), output)
    }

    pub fn new(
        config: ConfigMap,
        paths_and_devs: &HashMap<String, Device>,
    ) -> Result<Self, FatalError> {
        let path_and_info_or_error: Result<HashMap<_, _>, Error> = paths_and_devs
            .iter()
            .map(|(p, d)| Self::get_device_info(p.clone(), d))
            .collect();

        let path_and_info = path_and_info_or_error?;

        let input_mappings: HashMap<(String, ControllerInputEvent), OutputEvent> = config
            .into_iter()
            .map(|((p, i), m)| Self::make_mapping(p.clone(), i, m, &path_and_info[&p]))
            .collect();

        let builtins: HashMap<(String, ControllerInputEvent), OutputEvent> = paths_and_devs
            .iter()
            .map(|(p, _)| Self::make_sync_mapping(p.clone()))
            .collect();

        let mappings: HashMap<(String, ControllerInputEvent), OutputEvent> = input_mappings
            .into_iter()
            .chain(builtins.into_iter())
            .collect();

        Ok(EventMapping { mappings })
    }

    pub fn get_output_event(
        &self,
        path: &str,
        event: &InputEvent,
    ) -> Result<OutputEvent, NonFatalError> {
        let input_event = ControllerInputEvent::try_from(event)?;

        let output_event = self.mappings.get(&(path.into(), input_event));

        match output_event {
            Some(ev) => Ok(ev.clone_set_value(event.0.value())),
            None => Err(NonFatalError::from(format!(
                "No mapping for event type {:?}",
                event
            ))),
        }
    }

    pub fn list_output_events(&self) -> Vec<&OutputEvent> {
        self.mappings.iter().map(|(_, m)| m).collect()
    }
}
