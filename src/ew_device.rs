use std::{
    collections::{HashMap, HashSet},
    io::Error,
    path::Path,
};

use crate::ew_types::{AbsInfo, AbsoluteAxisType, Key};

pub struct Device(evdev::Device);

impl Device {
    pub fn open(path: impl AsRef<Path>) -> Result<Device, Error> {
        let raw_dev = evdev::Device::open(path)?;
        Ok(Device(raw_dev))
    }

    pub fn get_abs_state(&self) -> Result<HashMap<AbsoluteAxisType, AbsInfo>, Error> {
        let axis_states = self.0.get_abs_state()?.to_vec();

        let mut abs_info: HashMap<AbsoluteAxisType, AbsInfo> = HashMap::new();

        if let Some(all_axis) = self.0.supported_absolute_axes() {
            abs_info = all_axis
                .iter()
                .map(|axis| {
                    let axis_no = axis.0 as usize;
                    (
                        AbsoluteAxisType(axis),
                        AbsInfo(evdev::AbsInfo::new(
                            axis_states[axis_no].value,
                            axis_states[axis_no].minimum,
                            axis_states[axis_no].maximum,
                            axis_states[axis_no].fuzz,
                            axis_states[axis_no].flat,
                            axis_states[axis_no].resolution,
                        )),
                    )
                })
                .collect();
        }

        Ok(abs_info)
    }

    pub fn name(&self) -> Option<&str> {
        self.0.name()
    }

    pub fn supported_keys(&self) -> HashSet<Key> {
        let mut key_info: HashSet<Key> = HashSet::new();
        if let Some(key_attrs) = self.0.supported_keys() {
            key_info = key_attrs.iter().map(Key).collect();
        }

        key_info
    }

    pub fn grab(&mut self) -> Result<(), Error> {
        self.0.grab()
    }

    pub fn into_event_stream(self) -> Result<evdev::EventStream, Error> {
        self.0.into_event_stream()
    }
}
