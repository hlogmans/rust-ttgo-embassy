# TTGO T-Display ESP32 Interactive Text Demo

An embedded Rust application for the TTGO T-Display ESP32 board featuring interactive text movement controlled by physical buttons.

## Overview

This project demonstrates async embedded programming on the ESP32 microcontroller using the `esp-hal` framework and Embassy async runtime. It showcases integration of an ST7789 LCD display with button input handling to create an interactive graphical application.

## Hardware Requirements

- **TTGO T-Display** - ESP32 board with integrated 1.14" ST7789 LCD (135x240 pixels)
- **Built-in buttons:**
  - Button 1 (GPIO0) - Move text left
  - Button 2 (GPIO35) - Move text right

## Features

### Display Management ([display.rs](src/display.rs))
- **ST7789 LCD Driver** - 135x240 RGB565 display initialization
- **SPI Interface** - 40MHz communication with the display
- **Graphics Primitives:**
  - Text rendering with 10x20 monospace font
  - Line drawing
  - Rectangle drawing
  - Color fill operations
- **Automatic Backlight Control** - Enables backlight after successful initialization
- **Embedded Graphics Integration** - Uses the `embedded-graphics` crate for drawing operations

### Button Input Handling ([buttons.rs](src/buttons.rs))
- **Async Button Monitoring** - Non-blocking button state detection using Embassy tasks
- **Debounced Input** - Waits for press and release events
- **Thread-Safe State Management** - Uses critical sections and shared state via `Mutex<RefCell<i32>>`
- **Multiple Button Support** - Task pool allows monitoring up to 2 buttons concurrently
- **Configurable Direction** - Each button can set different movement values

### Main Application ([main.rs](src/bin/main.rs))
- **Interactive Text Display** - "Hello World ^_^;" text that moves horizontally
- **Button-Controlled Movement:**
  - Left button (GPIO0): Moves text left by 10 pixels per press
  - Right button (GPIO35): Moves text right by 10 pixels per press
- **Boundary Detection** - Text stays within display bounds
- **Visual Feedback:**
  - White text on black background
  - Green rectangle border around text
  - Yellow diagonal line across screen
- **Smooth Updates** - 50ms refresh rate (20 FPS)

## Technical Stack

- **Language:** Rust (no_std embedded environment)
- **Target:** Xtensa ESP32 (xtensa-esp32-none-elf)
- **Async Runtime:** Embassy 0.9.1
- **HAL:** esp-hal 1.0
- **RTOS:** esp-rtos 0.2.0 with Embassy support
- **Display Driver:** mipidsi 0.9.0
- **Graphics:** embedded-graphics 0.8.1

## Project Structure

```
src/
├── bin/
│   └── main.rs      # Main application entry point
├── buttons.rs       # Button input handling and state management
├── display.rs       # ST7789 display initialization and drawing functions
└── lib.rs          # Library root (module exports)
```

## Building and Flashing

### Prerequisites
- Rust toolchain for ESP32 (see `rust-toolchain.toml`)
- espflash tool for flashing the device

### Build Commands

```bash
# Build in release mode (optimized for size and performance)
cargo build --release

# Build and flash to device
cargo run --release
```

## Configuration

### Display Pins
- **MOSI:** GPIO19
- **SCK:** GPIO18
- **CS:** GPIO5
- **DC:** GPIO16
- **RST:** GPIO23
- **Backlight:** GPIO4

### Button Pins
- **Button 1:** GPIO0 (built-in button)
- **Button 2:** GPIO35 (built-in button with pull-up)

### Performance Settings
- **CPU Clock:** Maximum frequency
- **SPI Frequency:** 40 MHz
- **Optimization:** Size-optimized (`opt-level = "s"`)
- **LTO:** Fat (Link-Time Optimization enabled)

## Code Highlights

### Async Button Monitoring
Each button runs in its own Embassy task, waiting asynchronously for button events without blocking:
```rust
btn.wait_for_low().await;  // Wait for button press
// Update shared state
btn.wait_for_high().await; // Wait for button release
```

### Thread-Safe State Sharing
Direction changes are communicated via a globally accessible mutex:
```rust
pub static DIRECTION: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(0));
```

### Display Abstraction
The display module provides a high-level API hiding embedded-graphics complexity:
- `clear_color()` - Fill screen with solid color
- `draw_text()` - Render text with automatic styling
- `draw_line()` - Draw lines with specified color
- `draw_rectangle()` - Draw rectangle outlines

## License

[Add your license information here]

## References

- [esp-hal examples](https://github.com/esp-rs/esp-hal/tree/esp-hal-v~1.0/examples)
- [TTGO T-Display specifications](https://github.com/Xinyuan-LilyGO/TTGO-T-Display)
