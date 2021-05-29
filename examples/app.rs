use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use lvgl;
use lvgl::display::{Display, DisplayBuffer, DisplayDriver};

type ColorSpace = Rgb565;

fn main() {
    let mut embedded_graphics_display: SimulatorDisplay<ColorSpace> = SimulatorDisplay::new(
        Size::new(lvgl_sys::LV_HOR_RES_MAX, lvgl_sys::LV_VER_RES_MAX),
    );

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("App Example", &output_settings);

    // LVGL usage
    lvgl::init();

    let mut display_diver: DisplayDriver<ColorSpace> =
        DisplayDriver::new(DisplayBuffer::new(), |pixels| {
            // Here we draw to the external display
            let _ = embedded_graphics_display.draw_iter(pixels);
        });
    let _display = lvgl::disp_drv_register(&mut display_diver).unwrap();
}
