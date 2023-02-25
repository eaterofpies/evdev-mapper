use std::collections::HashMap;
use evdev::Device;

fn print_list_item(path: &str, phy_path: &str, name: &str){
    println!("| {0: <20} | {1:<30} | {2:}",path, phy_path, name)
}

pub fn list(){
    let devices = evdev::enumerate().collect::<HashMap<_,_>>();
    // readdir returns them in reverse order from their eventN names for some reason
    print_list_item("path", "physical path", "name");
    print_list_item("--------------------", "------------------------------", "----");

    for path_and_dev in devices.iter() {
        let path = path_and_dev.0;
        let dev = path_and_dev.1;
        print_list_item(
            path.as_os_str().to_string_lossy().as_ref(),
            dev.physical_path().unwrap_or("Unknown Path") ,
            dev.name().unwrap_or("Unnamed device")
        );
    }
}

pub fn properties(path: String){
    let device = Device::open(path).unwrap();
    println!("Device: {}", device.name().unwrap_or("unknown"));

    if let Some(all_axis) = device.supported_keys(){
        println!("Keys:");
        for axis in all_axis.iter(){
            println!("\t{:?}", axis)
        }
    }

    if let Some(all_axis) = device.supported_absolute_axes(){
        println!("Absolute axis:");
        for axis in all_axis.iter(){
            println!("\t{:?}", axis)
        }
    }
}

pub fn open(paths: Vec<String>) -> HashMap<String, Device>{
    paths
        .iter()
        .map(|path| (path.to_owned(), Device::open(path).unwrap()))
        .collect()
}
