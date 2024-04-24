use std::{collections::HashMap, sync::Mutex};

lazy_static! {
    static ref EVENTS: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
}

pub fn clear_events() {
    EVENTS.lock().unwrap().clear();
}

pub fn record_event<T: Into<String>>(event: T, n: i32) {
    let event_name = event.into();
    let mut events_lock = EVENTS.lock();
    let events = events_lock.as_mut().unwrap();

    events
        .entry(event_name)
        .and_modify(|e| *e += n)
        .or_insert(n);
}

pub fn get_event_count<T: Into<String>>(event: T) -> i32 {
    let events = EVENTS.lock().unwrap();
    events.get(&event.into()).map_or(0, |e| *e)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn clone_events() -> HashMap<String, i32> {
    EVENTS.lock().unwrap().clone()
}

pub fn load_events(events: HashMap<String, i32>) {
    EVENTS.lock().unwrap().clear();
    events.iter().for_each(|(k, v)| {
        EVENTS.lock().unwrap().insert(k.to_string(), *v);
    });
}
