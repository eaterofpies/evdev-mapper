
use std::error::Error;

use futures::stream::{FuturesUnordered,StreamExt};
use std::collections::HashMap;
use evdev::{Device, EventStream, InputEvent};

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

// fn list_dev_properties(path: String){

//     let device = Device::open(path).unwrap();

//     println!("{:?}", device.supported_keys());
//     let name = device.name();
//     let path = device.to_string();
//     println!("using device {} ({})", path, name.unwrap());
// }


fn open_devices(paths: Vec<String>) -> HashMap<String, Device>{
    paths
        .iter()
        .map(|path| (path.to_owned(), Device::open(path).unwrap()))
        .collect()
}

async fn next_event_with_meta(path: &String, stream: &mut EventStream) -> (String, InputEvent) {
    (path.to_owned(), stream.next_event().await.unwrap())
}

async fn combine_devices(devices: HashMap<String, Device>)-> Result<(), Box<dyn Error>>{
    let mut streams: HashMap<_,_> = devices
        .into_iter()
        .map(|(p, d)| (p, d.into_event_stream().unwrap()))
        .collect();

    loop {
        let mut futures = FuturesUnordered::from_iter(
            streams
                .iter_mut()
                .map(|(p, s)| next_event_with_meta(p, s)));

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
        let paths_and_devs = open_devices(paths);
        combine_devices(paths_and_devs).await
    }
}
