use std::{
    collections::{HashMap, HashSet},
    io::Error,
};

use crate::{
    ew_device::Device,
    ew_types::{AbsInfo, AbsoluteAxisType, KeyCode},
};

pub struct DeviceInfo {
    pub axis_info: HashMap<AbsoluteAxisType, AbsInfo>,
    pub key_info: HashSet<KeyCode>,
}

pub fn get_device_info(device: &Device) -> Result<DeviceInfo, Error> {
    let key_info: HashSet<KeyCode> = device.supported_keys();
    let axis_info = device.get_abs_state()?;

    Ok(DeviceInfo {
        axis_info,
        key_info,
    })
}

fn print_list_item(path: &str, phy_path: &str, name: &str) {
    println!("| {0: <20} | {1:<30} | {2:}", path, phy_path, name)
}

pub fn list() {
    let devices = evdev::enumerate().collect::<HashMap<_, _>>();
    // readdir returns them in reverse order from their eventN names for some reason
    print_list_item("path", "physical path", "name");
    print_list_item(
        "--------------------",
        "------------------------------",
        "----",
    );

    for path_and_dev in devices.iter() {
        let path = path_and_dev.0;
        let dev = path_and_dev.1;
        print_list_item(
            path.as_os_str().to_string_lossy().as_ref(),
            dev.physical_path().unwrap_or("Unknown Path"),
            dev.name().unwrap_or("Unnamed device"),
        );
    }
}

fn print_properties(device: &Device) -> Result<(), Error> {
    println!("Device: {}", device.name().unwrap_or("unknown"));

    let all_axis = device.supported_keys();
    println!("Keys:");
    for axis in all_axis.iter() {
        println!("\t{:?}", axis)
    }

    let abs_info = device.get_abs_state()?;

    println!("Absolute axis:");
    for (k, v) in abs_info.iter() {
        println!("\t{:?}: {:?}", k, v)
    }

    Ok(())
}

fn open_device(path: &String) -> Result<Device, Error> {
    let mut device = Device::open(path)?;

    // Grab the device to stop duplicate events from multiple devices
    device.grab()?;

    print_properties(&device)?;
    Ok(device)
}

pub fn properties(path: String) -> Result<(), Error> {
    let device = Device::open(path)?;
    print_properties(&device)?;
    Ok(())
}

pub fn open_devices(paths: HashSet<String>) -> Result<HashMap<String, Device>, Error> {
    let mut devices: HashMap<String, Device> = HashMap::new();

    for path in paths {
        let device = open_device(&path)?;
        devices.insert(path.clone(), device);
    }

    Ok(devices)
}
