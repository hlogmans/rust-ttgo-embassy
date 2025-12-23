#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;

use esp_hal::gpio::{Input, InputConfig, Pull};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;

use log::info;

use ttgo_3::{display, buttons};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.1.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // GPIO

    // Display: initialize via helper module
    let mut buffer = [0_u8; 512];
    let disp_pins = display::DisplayPeripherals {
        spi2: peripherals.SPI2,
        gpio_mosi: peripherals.GPIO19,
        gpio_sck: peripherals.GPIO18,
        gpio_cs: peripherals.GPIO5,
        gpio_dc: peripherals.GPIO16,
        gpio_rst: peripherals.GPIO23,
        gpio_bl: peripherals.GPIO4,
    };
    let mut display = display::init(disp_pins, &mut buffer);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Text
    let char_w: i32 = 10;
    //let char_h = 20;
    let text = "Hello World ^_^;";
    let mut text_x = display::W / 2;
    let text_y = display::H / 2;

    let btn1 = Input::new(peripherals.GPIO0, InputConfig::default());
    let btn2 = Input::new(peripherals.GPIO35, InputConfig::default().with_pull(Pull::Up));

    // Spawn button monitor tasks with proper error handling
    if let Err(e) = spawner.spawn(buttons::monitor_button(btn1, -10, "Click left")) {
        info!("Failed to spawn btn1 monitor task: {:?}", e);
    }
    if let Err(e) = spawner.spawn(buttons::monitor_button(btn2, 10, "Click right")) {
        info!("Failed to spawn btn2 monitor task: {:?}", e);
    }

    display.clear_color(Rgb565::BLACK);

    let mut firstttime = true;

    loop {
        Timer::after(Duration::from_millis_floor(50)).await;

        //display.clear_color(colors[counter % colors.len()]);


        let change = buttons::get_direction();

        if change != 0 || firstttime {
            firstttime = false;
            info!("Change detected: {}", change);
             // remove old text
            display.draw_text(text, text_x, text_y, true);
            display.draw_rectangle(
                text_x - 2,
                text_y - 2,
                (text.len() as u32) * char_w as u32 + 4,
                24,
                Rgb565::BLACK,
            );

            text_x = if text_x + change < 0 {
                0
            } else if text_x + change > display::W - char_w {
                display::W - char_w
            } else {
                text_x + change
            };
            // draw new text
            display.draw_text(text, text_x, text_y, false);

            // draw surrounding rectangle
            display.draw_rectangle(
                text_x - 2,
                text_y - 2,
                (text.len() as u32) * char_w as u32 + 4,
                24,
                Rgb565::GREEN,
            );

        }

        display.draw_line(0, 0, display::W - 1, display::H - 1, Rgb565::YELLOW);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v~1.0/examples
}
