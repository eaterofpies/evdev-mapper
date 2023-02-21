
use std::error::Error;

use futures::stream::{FuturesUnordered,StreamExt};
use std::collections::HashMap;

fn print_list_item(path: &str, phy_path: &str, name: &str){
    println!("| {0: <20} | {1:<30} | {2:}",path, phy_path, name)
}

fn list_dev_paths(){
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

fn find_dev_by_path(path: &str) -> evdev::Device {
    evdev::Device::open(path).unwrap()
}

async fn combine_devices(devices: Vec<evdev::Device>)-> Result<(), Box<dyn Error>>{
    for device in devices.iter(){
        println!("{:?}", device.supported_keys());
        let name = device.name();
        let path = device.to_string();
        println!("using device {} ({})", path, name.unwrap());
    }

    let mut streams: Vec<_> = devices.into_iter().map(|d| d.into_event_stream().unwrap()).collect();

    loop {
        let mut futures = FuturesUnordered::from_iter(streams.iter_mut().map(|s| s.next_event()));

        let ev = futures.next().await.unwrap();
        println!("{:?}", ev);
    }
}

#[tokio::main]
async fn main()-> Result<(), Box<dyn Error>> {
    use std::env;
    let args: Vec<String> = env::args().collect();
    println!("args {:?}", args);
    if args[1] == "--list" {
        list_dev_paths();
        Ok(())
    }
    else {
        let paths = args.as_slice()[1..].to_vec();
        let devs:Vec<_> = paths.iter().map(|path| find_dev_by_path(path)).collect();
        combine_devices(devs).await
    }
}
