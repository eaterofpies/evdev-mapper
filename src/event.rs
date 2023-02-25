use std::ops::Deref;
use evdev::AbsoluteAxisType;
use std::hash::{Hash, Hasher};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AbsAxis(pub AbsoluteAxisType);

impl Deref for AbsAxis {
    type Target = AbsoluteAxisType;

    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl Eq for AbsAxis {
}


impl Hash for AbsAxis {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.0.hash(state)
    }
}

impl PartialEq for AbsAxis {
    fn eq(&self, other: &AbsAxis) -> bool{
        self.0 == other.0
    }
}


#[derive(Debug, Deserialize)]
pub struct Key(pub evdev::Key);

impl Deref for Key {
    type Target = evdev::Key;

    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl Eq for Key {
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}


impl PartialEq for Key {
    fn eq(&self, other: &Key) -> bool{
        self.0 == other.0
    }
}
