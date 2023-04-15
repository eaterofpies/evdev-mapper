mod args;
mod config;
mod device;
mod ew_device;
mod ew_types;
mod ew_uinput;
mod mapping;
mod uinput;

use args::Mode;
use clap::Parser;
use config::{ConfigMap, ControllerEvent};
use evdev::InputEventKind;
use ew_device::Device;
use ew_types::{AbsoluteAxisType, EventStream, InputEvent, KeyCode, Synchronization};
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
    let mode = args.mode;
    let config_path = args.config;

    match mode {
        Mode::Devices => {
            device::list();
            Ok(())
        }
        Mode::Properties => {
            match args.device {
                Some(device_path) => device::properties(device_path)?,
                None => println!("Device must be set in 'properties' mode."),
            }
            Ok(())
        }
        Mode::Run => {
            let config = config::read(&config_path);
            match config {
                Ok(c) => {
                    run(c).await?;
                }
                Err(e) => {
                    println!("Failed to read config file '{:}'. {:}.", config_path, e);
                }
            };

            Ok(())
        }
    }
}

async fn run(config: ConfigMap) -> Result<(), Box<dyn Error>> {
    let paths: Vec<_> = config.iter().map(|(p, _m)| p.to_owned()).collect();
    let paths_and_devs = device::open_devices(paths)?;

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

    let mut output_device = new_device(&mappings)?;

    loop {
        // Setup futures for the event sources
        let mut futures = FuturesUnordered::from_iter(
            streams.iter_mut().map(|(p, s)| next_event_with_meta(p, s)),
        );

        // wait for an event
        if let Some((path, event)) = futures.next().await {
            let result = interpret_event(&path, &event, &mappings);

            let result = match result {
                Ok(ev) => {
                    println!("writing event {:?}", ev);
                    output_device.emit(&[ev]).map_err(NonFatalError::Io)
                }
                Err(err) => Err(err),
            };

            match result {
                Ok(_) => (),
                Err(e) => println!("{:?}", e),
            }
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
) -> std::result::Result<OutputEvent, NonFatalError> {
    // Make a ControllerEvent from the input
    let input_event = match event.kind() {
        InputEventKind::AbsAxis(a) => Ok(ControllerEvent::AbsAxis(AbsoluteAxisType(a))),
        InputEventKind::Key(a) => Ok(ControllerEvent::Key(KeyCode(a))),
        InputEventKind::Synchronization(a) => {
            Ok(ControllerEvent::Synchronization(Synchronization(a)))
        }
        _ => Err(NonFatalError::Str(format!(
            "No handler for event type {:?}",
            event
        ))),
    }?;

    // Ignore sync events for now as the mapping isn't set up.
    let output_event = event_mappings.get(path).and_then(|m| m.get(&input_event));

    match output_event {
        Some(oe) => Ok(oe.clone_set_value(event.0.value())),
        None => Err(NonFatalError::Str(format!(
            "No mapping for event type {:?}",
            event
        ))),
    }
}
