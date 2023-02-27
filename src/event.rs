use std::ops::Deref;
use std::hash::{Hash, Hasher};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct AbsoluteAxisType(pub evdev::AbsoluteAxisType);

impl Deref for AbsoluteAxisType {
    type Target = evdev::AbsoluteAxisType;

    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl Eq for AbsoluteAxisType {
}


impl Hash for AbsoluteAxisType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.0.hash(state)
    }
}

impl PartialEq for AbsoluteAxisType {
    fn eq(&self, other: &AbsoluteAxisType) -> bool{
        self.0 == other.0
    }
}


#[derive(Clone, Debug, Deserialize)]
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
