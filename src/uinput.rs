use crate::mapping::{EventMapping, OutputEvent};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, UinputAbsSetup};
use std::io::Error;

// Get a list of all of the possible outputs for the new device
fn get_output_actions(dev_mappings: &EventMapping) -> Vec<&OutputEvent> {
    let mut output_actions: Vec<&OutputEvent> = Vec::new();
    for mappings in dev_mappings.values() {
        let dev_events: Vec<_> = mappings.iter().map(|(_i, o)| o).collect();
        output_actions.extend(dev_events);
    }

    output_actions
}

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
            OutputEvent::Key(a) => keys.insert(a.0),
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

    builder.build()
}

pub fn new_device(dev_mappings: &EventMapping) -> Result<VirtualDevice, Error> {
    let output_actions = get_output_actions(dev_mappings);
    let (all_axis, keys) = make_uniput_config(output_actions);
    let mut device = build_device(all_axis, keys)?;

    for path in device.enumerate_dev_nodes_blocking()? {
        let path = path?;
        println!("Available as {}", path.display());
    }

    Ok(device)
}
