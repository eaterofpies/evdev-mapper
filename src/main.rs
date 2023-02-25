

use clap::Parser;
use evdev::{Device, EventStream, AbsoluteAxisType, Key, InputEvent, InputEventKind};
use futures::stream::{FuturesUnordered,StreamExt};

use serde::{Deserialize};

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::ops::Deref;


#[derive(Debug, Deserialize)]
struct Config {
    devices: Vec<DeviceConfig>,
}

#[derive(Debug, Deserialize)]
struct DeviceConfig {
    path: String,
    mappings: Vec<EventMapping>,
}

#[derive(Debug, Deserialize)]
struct EventMapping {
    input_event: ControllerEvent,
    output_event: ControllerEvent,
}


#[derive(Debug, Deserialize)]
struct LocAbsAxis(AbsoluteAxisType);

#[derive(Debug, Deserialize)]
struct LocKey(Key);

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
enum ControllerEvent{
    AbsAxis(LocAbsAxis),
    Key(LocKey),
}

impl Deref for LocAbsAxis {
    type Target = AbsoluteAxisType;

    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl Eq for LocAbsAxis {
}


impl Hash for LocAbsAxis {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.0.hash(state)
    }
}

impl PartialEq for LocAbsAxis {
    fn eq(&self, other: &LocAbsAxis) -> bool{
        self.0 == other.0
    }
}


impl Deref for LocKey {
    type Target = Key;

    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl Eq for LocKey {
}

impl Hash for LocKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}


impl PartialEq for LocKey {
    fn eq(&self, other: &LocKey) -> bool{
        self.0 == other.0
    }
}


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

async fn combine_devices(devices: HashMap<String, Device>, device_mappings: HashMap<String, HashMap<ControllerEvent,ControllerEvent>>)-> Result<(), Box<dyn Error>>{
    // Setup event streams
    let mut streams: HashMap<_,_> = devices
        .into_iter()
        .map(|(p, d)| (p, d.into_event_stream().unwrap()))
        .collect();

    loop {
        // Setup futures for the event sources
        let mut futures = FuturesUnordered::from_iter(
            streams
                .iter_mut()
                .map(|(p, s)| next_event_with_meta(p, s)));

        // wait for an event
        let path_and_event = futures.next().await.unwrap();
        let path = &path_and_event.0;
        let event = &path_and_event.1;

        // Make a ControllerEvent from the input
        let maybe_input_event = match event.kind(){
            InputEventKind::AbsAxis(a) => Option::from(ControllerEvent::AbsAxis(LocAbsAxis(a))),
            InputEventKind::Key(a) => Option::from(ControllerEvent::Key(LocKey(a))),
            _ => None,
        };

        // Interpret the event
        if let Some(input_event) = maybe_input_event {
            if let Some(event_mapping) = device_mappings.get(path){
                if let Some(output_event) = event_mapping.get(&input_event) {
                    println!("event = {:?}", output_event)
                }
            }
        }


        // Map the event
        //config[path]
        //println!("{:?}", path_and_event);
    }
}

fn mappings_to_map(mappings: Vec<EventMapping>) -> HashMap<ControllerEvent,ControllerEvent>{
    let map:HashMap<_,_> = mappings.into_iter().map(|m| (m.input_event, m.output_event)).collect();
    map
}

fn read_config() -> HashMap<String, HashMap<ControllerEvent,ControllerEvent>>{
    let file = File::open("device.conf").unwrap();
    let config: Config = serde_yaml::from_reader(file).expect("Could not read values.");

    let config_map: HashMap<_,_> = config.devices.into_iter().map(|d| (d.path, mappings_to_map(d.mappings))).collect();
    println!("{:?}", config_map);
    config_map
}

async fn run(config: HashMap<String, HashMap<ControllerEvent,ControllerEvent>>)-> Result<(), Box<dyn Error>>{
    let paths: Vec<_>  = config.iter().map(|(p, _m)| p.to_owned()).collect();
    let paths_and_devs = open_devices(paths);

    combine_devices(paths_and_devs, config).await
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
        let config = read_config();
        run(config).await.unwrap();
        Ok(())
    }
}
