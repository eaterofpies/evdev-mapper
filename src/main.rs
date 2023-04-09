mod args;
mod config;
mod device;
mod event;
mod mapping;
mod uinput;

use args::Mode;
use clap::Parser;
use config::{ConfigMap, ControllerEvent};
use evdev::{Device, EventStream, InputEvent, InputEventKind};
use event::{AbsoluteAxisType, Key, Synchronization};
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
    let mode = args.mode.unwrap_or(Mode::Run);

    match mode {
        Mode::Devices => {
            device::list();
            Ok(())
        }
        Mode::Properties => {
            device::properties(args.device.unwrap());
            Ok(())
        }
        Mode::Run => {
            let config = config::read();
            run(config).await.unwrap();
            Ok(())
        }
    }
}

async fn run(config: ConfigMap) -> Result<(), Box<dyn Error>> {
    let paths: Vec<_> = config.iter().map(|(p, _m)| p.to_owned()).collect();
    let paths_and_devs = device::open_devices(paths);

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
        let (path, event) = futures.next().await.unwrap();
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

async fn next_event_with_meta(path: &String, stream: &mut EventStream) -> (String, InputEvent) {
    (path.to_owned(), stream.next_event().await.unwrap())
}

fn interpret_event(
    path: &String,
    event: &InputEvent,
    event_mappings: &EventMapping,
) -> std::result::Result<evdev::InputEvent, NonFatalError> {
    // Make a ControllerEvent from the input
    let maybe_input_event = match event.kind() {
        InputEventKind::AbsAxis(a) => Some(ControllerEvent::AbsAxis(AbsoluteAxisType(a))),
        InputEventKind::Key(a) => Some(ControllerEvent::Key(Key(a))),
        InputEventKind::Synchronization(a) => {
            Some(ControllerEvent::Synchronization(Synchronization(a)))
        }
        _ => None,
    };

    //    maybe_input_event.and_then(|e| )
    // Interpret the event
    //    if let Some(input_event) = maybe_input_event {
    let output_event = maybe_input_event
        .and_then(|input_event| event_mappings.get(path).and_then(|m| m.get(&input_event)));

    match output_event {
        Some(OutputEvent::AbsAxis(a)) => Ok(InputEvent::new(
            evdev::EventType::ABSOLUTE,
            a.axis_type.0 .0,
            event.value(),
        )),
        Some(OutputEvent::Key(k)) => Ok(InputEvent::new(
            evdev::EventType::KEY,
            k.code(),
            event.value(),
        )),
        Some(OutputEvent::Synchronization(_a)) => Ok(InputEvent::new(
            evdev::EventType::SYNCHRONIZATION,
            event.code(),
            event.value(),
        )),
        None => Err(NonFatalError::Str(format!(
            "No handler for event type {:?}",
            event
        ))),
    }
}
