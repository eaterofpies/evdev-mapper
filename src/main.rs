

use clap::Parser;
use evdev::{Device, EventStream, InputEvent};
use futures::stream::{FuturesUnordered,StreamExt};
use std::collections::HashMap;
use std::error::Error;

/// Combine multiple input devices into a single virtual device
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// mode to run in [devices, properties, run]
    #[arg(short, long)]
    mode: Option<String>,

    /// Device (required in properties mode)
    #[arg(short, long)]
    device: Option<String>
 }


fn print_list_item(path: &str, phy_path: &str, name: &str){
    println!("| {0: <20} | {1:<30} | {2:}",path, phy_path, name)
}

fn list_devices(){
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

fn list_properties(path: String){
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

async fn run(device: &str)-> Result<(), Box<dyn Error>>{
    let devices = Vec::from([device.to_owned()]);
    let paths_and_devs = open_devices(devices);

    combine_devices(paths_and_devs).await
}

#[tokio::main]
async fn main()-> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mode = args.mode.as_deref().unwrap_or("run");
    if mode == "devices" {
        list_devices();
        Ok(())
    }
    else if mode == "properties" {
        list_properties(args.device.unwrap());
        Ok(())
    }
    else {
        run(args.device.as_deref().unwrap()).await.unwrap();
        Ok(())
    }
}
