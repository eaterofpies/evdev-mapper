use evdev::EventType;
use serde::Deserialize;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct AbsoluteAxisType(pub evdev::AbsoluteAxisCode);

impl Deref for AbsoluteAxisType {
    type Target = evdev::AbsoluteAxisCode;

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

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct KeyCode(pub evdev::KeyCode);

impl Deref for KeyCode {
    type Target = evdev::KeyCode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Eq for KeyCode {}

impl Hash for KeyCode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl PartialEq for KeyCode {
    fn eq(&self, other: &KeyCode) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Copy)]
pub struct AbsInfo(pub evdev::AbsInfo);

impl AbsInfo {
    pub fn clone_set_value(&self, value: i32) -> Self {
        AbsInfo(evdev::AbsInfo::new(
            value,
            self.0.minimum(),
            self.0.maximum(),
            self.0.fuzz(),
            self.0.flat(),
            self.0.resolution(),
        ))
    }
}

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
pub struct Synchronization(pub evdev::SynchronizationCode);

impl Hash for Synchronization {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0 .0.hash(state)
    }
}

impl PartialEq for Synchronization {
    fn eq(&self, other: &Synchronization) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Debug)]
pub struct InputEvent(pub evdev::InputEvent);
impl InputEvent {
    pub fn new(type_: EventType, code: u16, value: i32) -> Self {
        Self(evdev::InputEvent::new(type_.0, code, value))
    }

    pub fn kind(&self) -> evdev::EventSummary {
        self.0.destructure()
    }
}

pub struct EventStream(pub evdev::EventStream);

impl EventStream {
    pub async fn next_event(&mut self) -> Result<InputEvent, std::io::Error> {
        let result = self.0.next_event().await?;
        Ok(InputEvent(result))
    }
}
