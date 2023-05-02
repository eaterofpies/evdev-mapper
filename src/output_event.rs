use crate::{
    config::FilteredKeyMapping,
    ew_types::{AbsInfo, AbsoluteAxisType, InputEvent, KeyCode},
};

#[derive(Clone, Debug)]
pub struct AbsAxisOutputEvent {
    pub axis_type: AbsoluteAxisType,
    pub axis_info: AbsInfo,
}

impl AbsAxisOutputEvent {
    pub fn clone_set_value(&self, value: i32) -> Self {
        AbsAxisOutputEvent {
            axis_type: self.axis_type,
            axis_info: self.axis_info.clone_set_value(value),
        }
    }

    pub fn to_evdev_event(&self) -> InputEvent {
        InputEvent::new(
            evdev::EventType::ABSOLUTE,
            self.axis_type.0 .0,
            self.axis_info.0.value(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct KeyOutputEvent {
    code: KeyCode,
    value: i32,
}

impl KeyOutputEvent {
    pub fn new(code: KeyCode, value: i32) -> Self {
        KeyOutputEvent { code, value }
    }

    pub fn code(&self) -> KeyCode {
        self.code
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn to_evdev_event(&self) -> InputEvent {
        InputEvent::new(evdev::EventType::KEY, self.code().0 .0, self.value())
    }
}

impl From<KeyCode> for KeyOutputEvent {
    fn from(k: KeyCode) -> Self {
        KeyOutputEvent { code: k, value: 0 }
    }
}

#[derive(Clone, Debug)]
pub struct SyncOutputEvent {
    code: u16,
    value: i32,
}

impl SyncOutputEvent {
    pub fn new() -> Self {
        Self { code: 0, value: 0 }
    }

    pub fn clone_set_value(&self, value: i32) -> Self {
        Self { code: 0, value }
    }

    pub fn code(&self) -> u16 {
        self.code
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn to_evdev_event(&self) -> InputEvent {
        InputEvent::new(evdev::EventType::SYNCHRONIZATION, self.code(), self.value())
    }
}

#[derive(Clone, Debug)]
pub struct FilteredAbsAxisOutputEvent {
    axis_type: AbsoluteAxisType,
    axis_info: AbsInfo,
    mappings: Vec<FilteredKeyMapping>,
}

impl FilteredAbsAxisOutputEvent {
    pub fn new(
        input_axis_type: AbsoluteAxisType,
        info: AbsInfo,
        mappings: Vec<FilteredKeyMapping>,
    ) -> Self {
        FilteredAbsAxisOutputEvent {
            axis_type: input_axis_type,
            axis_info: info,
            mappings,
        }
    }
    pub fn codes(&self) -> Vec<KeyCode> {
        self.mappings.iter().map(|f| f.key).collect()
    }

    pub fn clone_set_value(&self, value: i32) -> Self {
        FilteredAbsAxisOutputEvent {
            axis_type: self.axis_type,
            axis_info: self.axis_info.clone_set_value(value),
            mappings: self.mappings.clone(),
        }
    }

    fn mapping_to_evdev_event(&self, mapping: &FilteredKeyMapping) -> InputEvent {
        let axis_value = self.axis_info.0.value();
        let mut out_value = 0;
        if axis_value >= mapping.min && axis_value <= mapping.max {
            out_value = 1
        }

        InputEvent::new(evdev::EventType::KEY, mapping.key.0 .0, out_value)
    }

    pub fn to_evdev_events(&self) -> Vec<InputEvent> {
        self.mappings
            .iter()
            .map(|e| self.mapping_to_evdev_event(e))
            .collect()
    }
}
// Can't just use config directly as we need to clone the input axis info and values
#[derive(Clone, Debug)]
pub enum OutputEvent {
    AbsAxis(AbsAxisOutputEvent),
    Key(KeyOutputEvent),
    Synchronization(SyncOutputEvent),
    FilteredAbsAxis(FilteredAbsAxisOutputEvent),
}

impl OutputEvent {
    pub fn clone_set_value(&self, value: i32) -> Self {
        match self {
            OutputEvent::AbsAxis(a) => OutputEvent::AbsAxis(a.clone_set_value(value)),
            OutputEvent::Key(k) => OutputEvent::Key(KeyOutputEvent::new(k.code(), value)),
            OutputEvent::Synchronization(s) => {
                OutputEvent::Synchronization(s.clone_set_value(value))
            }
            OutputEvent::FilteredAbsAxis(f) => {
                OutputEvent::FilteredAbsAxis(f.clone_set_value(value))
            }
        }
    }

    pub fn to_evdev_events(&self) -> Vec<InputEvent> {
        match self {
            OutputEvent::AbsAxis(a) => vec![a.to_evdev_event()],
            OutputEvent::Key(k) => vec![k.to_evdev_event()],
            OutputEvent::Synchronization(s) => vec![s.to_evdev_event()],
            OutputEvent::FilteredAbsAxis(f) => f.to_evdev_events(),
        }
    }
}
