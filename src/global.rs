use crate::state::State;
use std::cell::Cell;
use std::ptr::NonNull;

thread_local! {
    pub static STATE: Cell<Option<NonNull<State>>> = const { Cell::new(None) };
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
