use std::{io::Error, path::PathBuf};

use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, InputEvent, UinputAbsSetup};

use crate::mapping::OutputEvent;

pub struct VirtualDevice(evdev::uinput::VirtualDevice);

fn make_uniput_config(
    output_actions: Vec<&OutputEvent>,
) -> (Vec<UinputAbsSetup>, AttributeSet<evdev::Key>) {
    // Need to build a list of all keys to pass to the builder
    // so we may as well extract the axis too
    let mut all_axis: Vec<UinputAbsSetup> = Vec::new();
    let mut keys: AttributeSet<evdev::Key> = AttributeSet::new();
    for event in output_actions {
        match event {
            OutputEvent::AbsAxis(a) => {
                let abs = UinputAbsSetup::new(a.axis_type.0, a.axis_info.0);
                all_axis.push(abs)
            }
            OutputEvent::Key(a) => keys.insert(a.code().0),
            OutputEvent::Synchronization(_) => (),
        }
    }

    (all_axis, keys)
}

fn build_device(
    all_axis: Vec<UinputAbsSetup>,
    keys: AttributeSet<evdev::Key>,
) -> Result<VirtualDevice, Error> {
    let builder = VirtualDeviceBuilder::new()?;
    let mut builder = builder.name("evdev-mapper gamepad").with_keys(&keys)?;

    for axis in all_axis {
        builder = builder.with_absolute_axis(&axis)?;
    }

    let device = builder.build()?;
    Ok(VirtualDevice(device))
}

fn wrangle_output_event(event: &OutputEvent) -> evdev::InputEvent {
    match event {
        OutputEvent::AbsAxis(a) => evdev::InputEvent::new(
            evdev::EventType::ABSOLUTE,
            a.axis_type.0 .0,
            a.axis_info.0.value(),
        ),
        OutputEvent::Key(k) => InputEvent::new(evdev::EventType::KEY, k.code().0 .0, k.value()),
        OutputEvent::Synchronization(s) => {
            InputEvent::new(evdev::EventType::SYNCHRONIZATION, s.code(), s.value())
        }
    }
}

impl VirtualDevice {
    pub fn new(output_events: Vec<&OutputEvent>) -> Result<Self, Error> {
        let (abs_axis, buttons) = make_uniput_config(output_events);
        build_device(abs_axis, buttons)
    }

    pub fn enumerate_dev_nodes_blocking(&mut self) -> Result<Vec<PathBuf>, Error> {
        let nodes = self.0.enumerate_dev_nodes_blocking()?;

        let mut paths: Vec<PathBuf> = Vec::new();
        for maybe_path in nodes {
            let path = maybe_path?;
            paths.push(path);
        }
        Ok(paths)
    }

    pub fn emit(&mut self, events: &[OutputEvent]) -> Result<(), Error> {
        let evdev_events: Vec<evdev::InputEvent> =
            events.iter().map(wrangle_output_event).collect();
        self.0.emit(&evdev_events)
    }
}
