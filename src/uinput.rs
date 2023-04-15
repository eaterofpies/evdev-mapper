use log::debug;

use crate::{
    ew_uinput::VirtualDevice,
    mapping::{EventMapping, OutputEvent},
};
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

pub fn new_device(dev_mappings: &EventMapping) -> Result<VirtualDevice, Error> {
    let output_actions = get_output_actions(dev_mappings);
    let mut device = VirtualDevice::new(output_actions)?;

    for path in device.enumerate_dev_nodes_blocking()? {
        debug!("Available as {}", path.display());
    }

    Ok(device)
}
