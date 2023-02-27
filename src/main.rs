mod args;
mod config;
mod device;
mod event;
mod mapping;
mod uinput;

use clap::Parser;
use config::{ConfigMap, ControllerEvent};
use evdev::{Device, EventStream, InputEvent, InputEventKind};
use event::{AbsoluteAxisType, Key};
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;
use std::error::Error;

use mapping::{make_mapping, EventMapping, OutputEvent};
use uinput::new_device;

#[derive(Debug)]
pub enum NonFatalError {
    Io(std::io::Error),
    Str(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = args::Args::parse();
    let mode = args.mode.as_deref().unwrap_or("run");

    if mode == "devices" {
        device::list();
        Ok(())
    } else if mode == "properties" {
        device::properties(args.device.unwrap());
        Ok(())
    } else {
        let config = config::read();
        run(config).await.unwrap();
        Ok(())
    }
}

async fn run(config: ConfigMap) -> Result<(), Box<dyn Error>> {
    let paths: Vec<_> = config.iter().map(|(p, _m)| p.to_owned()).collect();
    let paths_and_devs = device::open(paths);

    let mappings = make_mapping(&config, &paths_and_devs);

    combine_devices(paths_and_devs, mappings).await
}

async fn combine_devices(
    devices: HashMap<String, Device>,
    mappings: EventMapping,
) -> Result<(), Box<dyn Error>> {
    // Setup event streams
    let mut streams: HashMap<_, _> = devices
        .into_iter()
        .map(|(p, d)| (p, d.into_event_stream().unwrap()))
        .collect();

    let mut output_device = new_device(&mappings);

    loop {
        // Setup futures for the event sources
        let mut futures = FuturesUnordered::from_iter(
            streams.iter_mut().map(|(p, s)| next_event_with_meta(p, s)),
        );

        // wait for an event
        let path_and_event = futures.next().await.unwrap();
        let output_event = interpret_event(&path_and_event.0, &path_and_event.1, &mappings);

        let message = match output_event {
            Some(OutputEvent::AbsAxis(a)) => Ok(InputEvent::new(
                evdev::EventType::ABSOLUTE,
                a.axis_type.0 .0,
                path_and_event.1.value(),
            )),
            Some(OutputEvent::Key(k)) => Ok(InputEvent::new(
                evdev::EventType::KEY,
                k.code(),
                path_and_event.1.value(),
            )),
            None => Err(NonFatalError::Str(format!(
                "No handler for event type {:?}",
                path_and_event.1
            ))),
        };

        let result = match message {
            Ok(ev) => {
                println!("writing event {:?}", ev);
                let res = output_device.emit(&[ev]);
                match res {
                    Ok(a) => Ok(a),
                    Err(err) => Err(NonFatalError::Io(err)),
                }
            }
            Err(err) => Err(err),
        };

        if let Err(e) = result {
            println!("{:?}", e);
        }
    }
}

async fn next_event_with_meta(path: &String, stream: &mut EventStream) -> (String, InputEvent) {
    (path.to_owned(), stream.next_event().await.unwrap())
}

fn interpret_event(
    path: &String,
    event: &InputEvent,
    event_mappings: &EventMapping,
) -> Option<OutputEvent> {
    // Make a ControllerEvent from the input
    let maybe_input_event = match event.kind() {
        InputEventKind::AbsAxis(a) => Option::from(ControllerEvent::AbsAxis(AbsoluteAxisType(a))),
        InputEventKind::Key(a) => Option::from(ControllerEvent::Key(Key(a))),
        _ => None,
    };

    // Interpret the event
    if let Some(input_event) = maybe_input_event {
        if let Some(event_mapping) = event_mappings.get(path) {
            if let Some(output_event) = event_mapping.get(&input_event) {
                return Some(output_event.clone());
            }
        }
    }

    None
}
