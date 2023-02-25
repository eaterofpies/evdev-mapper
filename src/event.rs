use std::ops::Deref;
use evdev::{AbsoluteAxisType, Key};
use std::hash::{Hash, Hasher};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LocAbsAxis(pub AbsoluteAxisType);

#[derive(Debug, Deserialize)]
pub struct LocKey(pub Key);

impl Deref for LocAbsAxis {
    type Target = AbsoluteAxisType;

    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl Eq for LocAbsAxis {
}


impl Hash for LocAbsAxis {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.0.hash(state)
    }
}

impl PartialEq for LocAbsAxis {
    fn eq(&self, other: &LocAbsAxis) -> bool{
        self.0 == other.0
    }
}


impl Deref for LocKey {
    type Target = Key;

    fn deref(&self) -> &Self::Target{
        &self.0
    }
}

impl Eq for LocKey {
}

impl Hash for LocKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}


impl PartialEq for LocKey {
    fn eq(&self, other: &LocKey) -> bool{
        self.0 == other.0
    }
}
