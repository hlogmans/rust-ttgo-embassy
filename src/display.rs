#![allow(clippy::large_stack_frames)]

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::Drawable;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::{DrawTarget, Point, RgbColor, Primitive, Size};
use embedded_graphics::text::Text;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use mipidsi::interface::SpiInterface;
use mipidsi::{models::ST7789, options::ColorInversion, Builder};

// Re-export common sizes so callers can rely on one source of truth.
pub const W: i32 = 135;
pub const H: i32 = 240;

// Remark: the display has an offset. Thanks to Sidney, Koen and Jeroen for figuring this out!

// Raw peripherals required to build the display for the TTGO T-Display (ESP32 + ST7789)
pub struct DisplayPeripherals<'a> {
    pub spi2: esp_hal::peripherals::SPI2<'a>,
    pub gpio_mosi: esp_hal::peripherals::GPIO19<'a>,
    pub gpio_sck: esp_hal::peripherals::GPIO18<'a>,
    pub gpio_cs: esp_hal::peripherals::GPIO5<'a>,
    pub gpio_dc: esp_hal::peripherals::GPIO16<'a>,
    pub gpio_rst: esp_hal::peripherals::GPIO23<'a>,
    pub gpio_bl: esp_hal::peripherals::GPIO4<'a>,
}

// Initialize the ST7789 display and turn on backlight.
// Returns a DrawTarget that can be used with embedded-graphics.
pub fn init<'a>(
    pins: DisplayPeripherals<'a>,
    buffer: &'a mut [u8; 512],
) -> Display<impl DrawTarget<Color = Rgb565> + 'a> {
    let spi = Spi::new(
        pins.spi2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_mosi(pins.gpio_mosi)
    .with_sck(pins.gpio_sck);

    let cs = Output::new(pins.gpio_cs, Level::High, OutputConfig::default());
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let dc = Output::new(pins.gpio_dc, Level::Low, OutputConfig::default());

    // The display interface uses the provided temporary buffer for transfers.
    let di = SpiInterface::new(spi_device, dc, buffer);

    let mut delay = Delay::new();
    let rst = Output::new(pins.gpio_rst, Level::High, OutputConfig::default());

    // Backlight off during init; turn on after successful init.
    let mut backlight = Output::new(pins.gpio_bl, Level::Low, OutputConfig::default());

    let display = Builder::new(ST7789, di)
        .display_size(W as u16, H as u16)
        .display_offset(52, 40)
        .reset_pin(rst)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap();

    // Turn on backlight once the panel is initialized.
    backlight.set_high();

    // Wrap in our high-level Display facade
    Display { inner: display }
}

// Simple high-level facade hiding low-level embedded-graphics usage.
pub struct Display<D: DrawTarget<Color = Rgb565>> {
    inner: D,
}

impl<D: DrawTarget<Color = Rgb565>> Display<D> {
    // Fill the screen with a color.
    pub fn clear_color(&mut self, color: Rgb565) {
        let _ = self.inner.clear(color);
    }

    // Draw text at x,y using a default font and white color.
    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, clear: bool) {
        let style = MonoTextStyle::new(&FONT_10X20, if clear { Rgb565::BLACK } else { Rgb565::WHITE });
        let _ = Text::new(text, Point::new(x, y), style).draw(&mut self.inner);
    }

    // Draw a line with a given color.
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb565) {
        let line = Line::new(Point::new(x0, y0), Point::new(x1, y1))
            .into_styled(PrimitiveStyle::with_stroke(color, 1));
        let _ = line.draw(&mut self.inner);
    }

    // Expose screen size for convenience.
    pub fn size(&self) -> (i32, i32) { (W, H) }

    // Draw an axis-aligned rectangle outline.
    pub fn draw_rectangle(&mut self, x: i32, y: i32, width: u32, height: u32, color: Rgb565) {
        let rect = Rectangle::new(Point::new(x, y), Size::new(width, height))
            .into_styled(PrimitiveStyle::with_stroke(color, 1));

        let _ = rect.draw(&mut self.inner);
    }
}
