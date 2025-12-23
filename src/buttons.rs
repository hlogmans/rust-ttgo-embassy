use core::cell::RefCell;
use critical_section::Mutex;
use embassy_executor::task;
use log::info;

// Shared direction state, defaulting to 1.
pub static DIRECTION: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(0));


// Generic button monitor task.
// - On each press (low), log and set the shared direction to `set_to`.
// - Then wait for release (high) before looping.
#[task(pool_size = 2)]
pub async fn monitor_button(
    mut btn: esp_hal::gpio::Input<'static>,
    set_to: i32,
    log_msg: &'static str,
) {
    loop {
        btn.wait_for_low().await;
        critical_section::with(|cs| {
            info!("{}", log_msg);
            let mut dir = DIRECTION.borrow(cs).borrow_mut();
            *dir += set_to;
        });
        btn.wait_for_high().await;
    }
}

// High-level accessor used by main.rs.
// Reads the current direction value and resets it to 0.
pub fn get_direction() -> i32 {
    critical_section::with(|cs| {
        let borrowed = DIRECTION.borrow(cs);
        let value = *borrowed.borrow();
        *borrowed.borrow_mut() = 0;
        value
    })
}
