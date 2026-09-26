use crate::state::State;
use std::cell::{Cell, RefCell};
use std::ptr::NonNull;

#[derive(Default, Clone)]
pub struct PendingConfig {
    pub window_title: Option<String>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_visible: Option<bool>,
    pub window_icon: Option<(Vec<u8>, u32, u32)>,
}

thread_local! {
    pub static STATE: Cell<Option<NonNull<State>>> = const { Cell::new(None) };
    pub static PENDING_CONFIG: RefCell<PendingConfig> = RefCell::new(PendingConfig::default());
}

#[allow(unused)]
fn with_global<R>(state: &mut State, f: impl FnOnce() -> R) -> R {
    STATE.set(Some(NonNull::from(&mut *state)));
    let r = f();
    STATE.set(None);
    r
}

#[inline(always)]
pub fn ctx<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| {
        let mut p = s
            .get()
            .expect("no active context (call only inside the run/update fn)");
        f(unsafe { p.as_mut() })
    })
}
