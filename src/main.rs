mod args;
mod config;
mod device;
mod error;
mod ew_device;
mod ew_types;
mod ew_uinput;
mod mapping;
mod output_event;
mod uinput;

use args::Mode;
use clap::Parser;
use config::{ConfigMap, ControllerId};
use error::FatalError;
use ew_device::Device;
use ew_types::{EventStream, InputEvent};
use ew_uinput::VirtualDevice;
use futures::stream::{FuturesUnordered, StreamExt};
use log::{debug, error, warn};
use std::collections::{HashMap, HashSet};
use std::error::Error;

use mapping::EventMapping;
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
    let paths: HashSet<_> = config.iter().map(|((p, _i), _m)| p.clone()).collect();
    let paths_and_devs = device::open_devices(paths)?;

    let mappings = EventMapping::new(config, &paths_and_devs)?;

    let output_device = new_device(&mappings)?;

    combine_devices(paths_and_devs, mappings, output_device).await
}

fn make_stream(
    id: ControllerId,
    device: Device,
) -> Result<(ControllerId, EventStream), FatalError> {
    let dev = device.into_event_stream()?;
    Ok((id, dev))
}

async fn combine_devices(
    devices: HashMap<ControllerId, Device>,
    mappings: EventMapping,
    mut output_device: VirtualDevice,
) -> Result<(), Box<dyn Error>> {
    // Setup event streams
    let streams_or_error: Result<HashMap<_, _>, _> = devices
        .into_iter()
        .map(|(p, d)| make_stream(p, d))
        .collect();

    let mut streams = streams_or_error?;

    loop {
        // Setup futures for the event sources
        let mut futures = FuturesUnordered::from_iter(
            streams.iter_mut().map(|(p, s)| next_event_with_meta(p, s)),
        );

        let event = futures.next().await;
        let result = match event {
            // Futures.next returned something that was ok
            Some(Ok((id, event))) => process_single_event(id, event, &mappings, &mut output_device),

            // Futures.next returned something that was an error
            Some(Err(e)) => Err(e)?,

            // Futures.next returned nothing
            None => Ok(()),
        };

        match result {
            Ok(_) => (),
            Err(e) => warn!("{:?}", e),
        };
    }
}

async fn next_event_with_meta(
    id: &ControllerId,
    stream: &mut EventStream,
) -> Result<(ControllerId, InputEvent), FatalError> {
    let next_event = stream.next_event().await?;
    Ok((id.to_owned(), next_event))
}

fn process_single_event(
    id: ControllerId,
    event: InputEvent,
    mappings: &EventMapping,
    device: &mut VirtualDevice,
) -> Result<(), NonFatalError> {
    let event = mappings.get_output_event(&id, &event)?;
    debug!("writing event {:?}", event);
    device.emit(&[event]).map_err(NonFatalError::Io)
}
