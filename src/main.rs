mod args;
mod event;
mod config;
mod device;

use evdev::{Device, EventStream, InputEvent, InputEventKind};
use futures::stream::{FuturesUnordered,StreamExt};

use std::collections::HashMap;
use std::error::Error;

use event::{AbsAxis, Key};
use config::{ControllerEvent, ConfigMap};
use clap::Parser;


#[tokio::main]
async fn main()-> Result<(), Box<dyn Error>> {
    let args = args::Args::parse();
    let mode = args.mode.as_deref().unwrap_or("run");

    if mode == "devices" {
        device::list();
        Ok(())
    }
    else if mode == "properties" {
        device::properties(args.device.unwrap());
        Ok(())
    }
    else {
        let config = config::read();
        run(config).await.unwrap();
        Ok(())
    }
}

async fn run(config: ConfigMap)-> Result<(), Box<dyn Error>>{
    let paths: Vec<_>  = config.iter().map(|(p, _m)| p.to_owned()).collect();
    let paths_and_devs = device::open(paths);

    combine_devices(paths_and_devs, config).await
}

async fn combine_devices(devices: HashMap<String, Device>, mappings: ConfigMap)-> Result<(), Box<dyn Error>>{
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

        interpret_event(&path_and_event.0, &path_and_event.1, &mappings)
    }
}

async fn next_event_with_meta(path: &String, stream: &mut EventStream) -> (String, InputEvent) {
    (path.to_owned(), stream.next_event().await.unwrap())
}

fn interpret_event(path: &String, event: &InputEvent, device_mappings: &ConfigMap){
    // Make a ControllerEvent from the input
    let maybe_input_event = match event.kind(){
        InputEventKind::AbsAxis(a) => Option::from(ControllerEvent::AbsAxis(AbsAxis(a))),
        InputEventKind::Key(a) => Option::from(ControllerEvent::Key(Key(a))),
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

    //Map the event
    //config[path]
    //println!("{:?}", path_and_event);
}
