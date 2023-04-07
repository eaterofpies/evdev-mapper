use evdev::Device;
use std::collections::HashMap;

use crate::event::{AbsInfo, AbsoluteAxisType};

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

fn print_properties(device: &Device) {
    println!("Device: {}", device.name().unwrap_or("unknown"));

    if let Some(all_axis) = device.supported_keys() {
        println!("Keys:");
        for axis in all_axis.iter() {
            println!("\t{:?}", axis)
        }
    }

    let abs_info = get_abs_info(device);

    println!("Absolute axis:");
    for (k, v) in abs_info.iter() {
        println!("\t{:?}: {:?}", k, v)
    }
}

fn open_device(path: &String) -> Device {
    let mut device = Device::open(path).unwrap();
    device.grab().unwrap();
    print_properties(&device);
    device
}

pub fn get_abs_info(device: &Device) -> HashMap<AbsoluteAxisType, AbsInfo> {
    let axis_states = device.get_abs_state().unwrap().to_vec();

    let mut abs_info: HashMap<AbsoluteAxisType, AbsInfo> = HashMap::new();

    if let Some(all_axis) = device.supported_absolute_axes() {
        abs_info = all_axis
            .iter()
            .map(|axis| {
                let axis_no = axis.0 as usize;
                (
                    AbsoluteAxisType(axis),
                    AbsInfo(evdev::AbsInfo::new(
                        axis_states[axis_no].value,
                        axis_states[axis_no].minimum,
                        axis_states[axis_no].maximum,
                        axis_states[axis_no].fuzz,
                        axis_states[axis_no].flat,
                        axis_states[axis_no].resolution,
                    )),
                )
            })
            .collect();
    }

    abs_info
}

pub fn properties(path: String) {
    let device = Device::open(path).unwrap();
    print_properties(&device);
}

pub fn open_devices(paths: Vec<String>) -> HashMap<String, Device> {
    paths
        .iter()
        .map(|path| (path.to_owned(), open_device(path)))
        .collect()
}
