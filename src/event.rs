use serde::Deserialize;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Clone, Debug, Deserialize)]
pub struct AbsoluteAxisType(pub evdev::AbsoluteAxisType);

impl Deref for AbsoluteAxisType {
    type Target = evdev::AbsoluteAxisType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Eq for AbsoluteAxisType {}

impl Hash for AbsoluteAxisType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0 .0.hash(state)
    }
}

impl PartialEq for AbsoluteAxisType {
    fn eq(&self, other: &AbsoluteAxisType) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Key(pub evdev::Key);

impl Deref for Key {
    type Target = evdev::Key;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Eq for Key {}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Key) -> bool {
        self.0 == other.0
    }
}


#[derive(Clone, Copy)]
pub struct AbsInfo(pub evdev::AbsInfo);

impl fmt::Debug for AbsInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug = f.debug_struct("AbsInfo");
        debug.field("value", &self.0.value());
        debug.field("min", &self.0.minimum());
        debug.field("max", &self.0.maximum());
        debug.field("fuzz", &self.0.fuzz());
        debug.field("flat", &self.0.flat());
        debug.field("resolution", &self.0.resolution());
        debug.finish()
    }
}

#[derive(Clone, Debug, Deserialize, Eq)]
pub struct Synchronization(pub evdev::Synchronization);

impl Hash for Synchronization {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.0.hash(state)
    }
}

impl PartialEq for Synchronization {
    fn eq(&self, other: &Synchronization) -> bool {
        self.0 == other.0
    }
}
