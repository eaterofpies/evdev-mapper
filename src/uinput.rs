use log::debug;

use crate::{ew_uinput::VirtualDevice, mapping::EventMapping};
use std::io::Error;

pub fn new_device(dev_mappings: &EventMapping) -> Result<VirtualDevice, Error> {
    let output_actions = dev_mappings.list_output_events();
    let mut device = VirtualDevice::new(output_actions)?;

    for path in device.enumerate_dev_nodes_blocking()? {
        debug!("Available as {}", path.display());
    }

    Ok(device)
}
