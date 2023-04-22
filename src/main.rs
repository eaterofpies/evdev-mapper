mod args;
mod config;
mod device;
mod error;
mod ew_device;
mod ew_types;
mod ew_uinput;
mod mapping;
mod uinput;

use args::Mode;
use clap::Parser;
use config::{ConfigMap, ControllerInputEvent};
use ew_device::Device;
use ew_types::{EventStream, InputEvent};
use ew_uinput::VirtualDevice;
use futures::stream::{FuturesUnordered, StreamExt};
use log::{debug, error, warn};
use std::collections::HashMap;
use std::error::Error;

use mapping::{make_mapping, EventMapping, OutputEvent};
use uinput::new_device;

use crate::error::NonFatalError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

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
                None => log::error!("Device must be set in 'properties' mode."),
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
                    error!("Failed to read config file '{:}'. {:}.", config_path, e);
                }
            };

            Ok(())
        }
    }
}

async fn run(config: ConfigMap) -> Result<(), Box<dyn Error>> {
    let paths: Vec<_> = config.iter().map(|(p, _m)| p.to_owned()).collect();
    let paths_and_devs = device::open_devices(paths)?;

    let mappings = make_mapping(&config, &paths_and_devs)?;

    let output_device = new_device(&mappings)?;

    combine_devices(paths_and_devs, mappings, output_device).await
}

async fn combine_devices(
    devices: HashMap<String, Device>,
    mappings: EventMapping,
    mut output_device: VirtualDevice,
) -> Result<(), Box<dyn Error>> {
    // Setup event streams
    let mut streams: HashMap<_, _> = devices
        .into_iter()
        .map(|(p, d)| (p, d.into_event_stream().unwrap()))
        .collect();

    loop {
        // Setup futures for the event sources
        let mut futures = FuturesUnordered::from_iter(
            streams.iter_mut().map(|(p, s)| next_event_with_meta(p, s)),
        );

        let result = match futures.next().await {
            Some((path, event)) => process_single_event(path, event, &mappings, &mut output_device),
            None => Ok(()),
        };

        match result {
            Ok(_) => (),
            Err(e) => warn!("{:?}", e),
        };
    }
}

async fn next_event_with_meta(path: &String, stream: &mut EventStream) -> (String, InputEvent) {
    (path.to_owned(), stream.next_event().await.unwrap())
}

fn process_single_event(
    path: String,
    event: InputEvent,
    mappings: &EventMapping,
    device: &mut VirtualDevice,
) -> Result<(), NonFatalError> {
    let event = interpret_event(&path, &event, mappings)?;
    debug!("writing event {:?}", event);
    device.emit(&[event]).map_err(NonFatalError::Io)
}

fn interpret_event(
    path: &String,
    event: &InputEvent,
    event_mappings: &EventMapping,
) -> std::result::Result<OutputEvent, NonFatalError> {
    // Make a ControllerEvent from the input
    let input_event = ControllerInputEvent::try_from(event)?;

    // Ignore sync events for now as the mapping isn't set up.
    let output_event = event_mappings.get(path).and_then(|m| m.get(&input_event));

    match output_event {
        Some(oe) => Ok(oe.clone_set_value(event.0.value())),
        None => Err(NonFatalError::from(format!(
            "No mapping for event type {:?}",
            event
        ))),
    }
}
