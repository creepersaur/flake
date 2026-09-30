use std::cell::RefCell;
use std::rc::Rc;

/// # Timer
///
/// A timer that counts down each frame. Can be paused and extended.
/// Actions can be boundd to the timer that will run on completion.
///
/// Don't put the timer in the loop
///
/// Timers won't run by default, you must call `add_timer()` and pass in the timer
/// to add them to the timer list. Or call `spawn_timer(seconds)` and it will run immediately.
///
/// ```
/// let mut my_timer = add_timer(Timer::new(5.0));
/// // or
/// let mut my_timer = spawn_timer(5.0);
/// ```
///
/// ```
/// println!("Time left: {}", my_timer.time_left);
/// println!("Timer is paused: {}", my_timer.paused);
/// ```
///
/// ```
/// my_timer.extend(2.0); // extend by 2 seconds
///
/// if my_timer.completed() {
///     // timer is completed
/// }
///
/// if my_timer.ongoing() {
///     // timer is active
/// }
///
/// // Pause/Unpause a timer
/// my_timer.pause();
/// my_timer.unpause();
/// my_timer.set_paused(true);
/// my_timer.toggle_paused();
///
/// // Action will run if the timer is completed (must be called each frame)
/// my_timer.on_completion(|| {
///     println!("Timer was completed!");
/// })
///
/// // Reset the time left to the original (does not include `extend()`)
/// my_timer.reset();
///
/// // Stop the timer (marks as completed())
/// my_timer.finish();
/// ```
///
/// Once a timer is completed it will be removed from the timer list.
/// The timer will stop ticking and `on_completion()` will stop firing.
///
/// ```
/// // Re-add a timer to the timer list (REMEMBER TO RESET IT)
/// my_timer.reset();
/// add_timer(my_timer);
/// ```
#[derive(Default, Clone, PartialEq, PartialOrd)]
pub struct Timer(pub Rc<RefCell<TimerData>>);

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct TimerData {
    pub time_left: f32,
    pub paused: bool,
    pub is_active: bool,
    original_time: f32,
}

impl Timer {
    // Create a new timer using seconds
    pub fn new(seconds: f32) -> Self {
        Self(Rc::new(RefCell::new(TimerData {
            time_left: seconds,
            paused: false,
            is_active: false,
            original_time: seconds,
        })))
    }

    // Tick down the time left by `dt` (deltatime) only if it's unpaused
    pub fn tick(&mut self, dt: f32) {
        if !self.0.borrow().paused {
            self.0.borrow_mut().time_left -= dt;
        }
    }

    // Reset the timer to original time (does not include `extend()`)
    pub fn reset(&self) {
        let mut t = self.0.borrow_mut();
        t.time_left = t.original_time;
    }

    // Sets time left to zero (completed) which may also remove the timer from the timer list
    pub fn finish(&mut self) {
        self.0.borrow_mut().time_left = 0.0;
    }

    // Pause the timer
    pub fn pause(&mut self) {
        self.0.borrow_mut().paused = true;
    }

    // Unpause the timer
    pub fn unpause(&mut self) {
        self.0.borrow_mut().paused = false;
    }

    // Set whether timer is paused or not
    pub fn set_paused(&mut self, paused: bool) {
        self.0.borrow_mut().paused = paused;
    }

    // Toggle the timer paused/unpaused
    pub fn toggle_paused(&self) {
        let mut t = self.0.borrow_mut();
        t.paused = !t.paused;
    }

    // Extend (or decrease) the timer's time left
    pub fn extend(&mut self, seconds: f32) {
        self.0.borrow_mut().time_left += seconds;
    }

    // Check if timer is completed
    pub fn completed(&self) -> bool {
        self.0.borrow().time_left <= 0.0
    }

    // Check if timer is ongoing (not completed)
    pub fn ongoing(&self) -> bool {
        self.0.borrow().time_left > 0.0
    }

    // Check if timer is active (in the timer list)
    pub fn is_active(&self) -> bool {
        self.0.borrow().is_active
    }

    // Runs an action if timer is completed (only runs if timer is in timer list)
    pub fn on_completion(&self, mut action: impl FnMut()) {
        if self.completed() && self.0.borrow().is_active {
            action()
        }
    }
}
