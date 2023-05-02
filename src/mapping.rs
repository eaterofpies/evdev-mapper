use crate::{
    config::{self, ConfigMap, ControllerId, ControllerInputEvent, UniqueControllerEvent},
    device::{get_device_info, DeviceInfo},
    error::{FatalError, NonFatalError},
    ew_device::Device,
    ew_types::{AbsoluteAxisType, InputEvent, Synchronization},
    output_event::{
        AbsAxisOutputEvent, FilteredAbsAxisOutputEvent, KeyOutputEvent, OutputEvent,
        SyncOutputEvent,
    },
    util::rewrap,
};
use std::{collections::HashMap, io::Error};

pub struct EventMapping {
    mappings: HashMap<UniqueControllerEvent, OutputEvent>,
}

impl EventMapping {
    fn get_device_info(
        id: ControllerId,
        device: &Device,
    ) -> Result<(ControllerId, DeviceInfo), Error> {
        Ok((id, get_device_info(device)?))
    }

    fn make_abs_axis_mapping(
        device_info: &DeviceInfo,
        axis_type: AbsoluteAxisType,
        axis_event: config::AbsAxisEvent,
    ) -> Result<OutputEvent, FatalError> {
        let (_, axis_info) = device_info
            .axis_info
            .iter()
            .find(|(k, _v)| *k == &axis_type)
            .ok_or(format!(
                "Failed to find axis info for input axis {:?}",
                axis_type
            ))?;

        let output_event = match axis_event {
            config::AbsAxisEvent::AbsAxis(a) => OutputEvent::AbsAxis(AbsAxisOutputEvent {
                axis_type: a,
                axis_info: *axis_info,
            }),
            config::AbsAxisEvent::FilteredKeys(f) => OutputEvent::FilteredAbsAxis(
                FilteredAbsAxisOutputEvent::new(axis_type.clone(), *axis_info, f),
            ),
        };

        Ok(output_event)
    }

    fn make_mapping(
        mapping: config::EventMapping,
        device_info: &DeviceInfo,
    ) -> Result<OutputEvent, FatalError> {
        let output = match mapping {
            config::EventMapping::KeyEvent { input: _, output } => {
                OutputEvent::Key(KeyOutputEvent::new(output, 0))
            }
            config::EventMapping::AbsAxisEvent { input, output } => {
                Self::make_abs_axis_mapping(device_info, input, output)?
            }
        };

        Ok(output)
    }

    fn make_sync_mapping(id: ControllerId) -> (UniqueControllerEvent, OutputEvent) {
        let input = ControllerInputEvent::Synchronization(Synchronization(
            evdev::Synchronization::SYN_REPORT,
        ));
        let output = OutputEvent::Synchronization(SyncOutputEvent::new());
        (UniqueControllerEvent::new(id, input), output)
    }

    pub fn new(
        config: ConfigMap,
        paths_and_devs: &HashMap<ControllerId, Device>,
    ) -> Result<Self, FatalError> {
        let id_and_info_or_error: Result<HashMap<_, _>, Error> = paths_and_devs
            .iter()
            .map(|(p, d)| Self::get_device_info(p.clone(), d))
            .collect();

        let id_and_info = id_and_info_or_error?;

        let input_mappings_or_error: Result<HashMap<_, _>, FatalError> = config
            .into_iter()
            .map(|(ue, m)| rewrap(ue.clone(), Self::make_mapping(m, &id_and_info[&ue.id])))
            .collect();

        let input_mappings = input_mappings_or_error?;

        let builtins: HashMap<_, _> = paths_and_devs
            .iter()
            .map(|(i, _)| Self::make_sync_mapping(i.clone()))
            .collect();

        let mappings: HashMap<_, _> = input_mappings
            .into_iter()
            .chain(builtins.into_iter())
            .collect();

        Ok(EventMapping { mappings })
    }

    pub fn get_output_event(
        &self,
        id: &ControllerId,
        event: &InputEvent,
    ) -> Result<OutputEvent, NonFatalError> {
        let input_event = ControllerInputEvent::try_from(event)?;

        let output_event = self
            .mappings
            .get(&UniqueControllerEvent::new(id.clone(), input_event));

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
