use super::Lvgl;
use super::Screen;
use alloc::boxed::Box;

use core::{
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr,
};

use embedded_graphics::{draw_target::DrawTarget, prelude::*, primitives::Rectangle};

//use super::{Obj, Screen};

// This gives us "pub type PixelColor = embedded_graphics_core::pixel_color::Rgb565;" with the right color
include!(concat!(env!("OUT_DIR"), "/generated-color-settings.rs"));

/// `Display` represents a display for Lvgl
/// Limitations:
/// * No async drawing, no double buffering
/// * No color conversion. lv_conf.h specifies what embedded_graphics display you can use
/// * Resources are leaked when `Display` is dropped

pub struct Display<T> {
    // We box because we need stable addresses
    display: Box<T>,
    pub(crate) disp: &'static mut lvgl_sys::lv_disp_t,
}

unsafe impl<T: Send> Send for Display<T> {}

impl<T: DrawTarget<Color = PixelColor> + OriginDimensions> Display<T> {
    /// Pass in the drawing buffer. See https://docs.lvgl.io/master/porting/display.html
    /// PixelColor is aliased to the color type configured by lv_conf.h
    /// 1/10th of the screen size is recommended for the size.
    // Note that we take references, because we want to be able to take special
    // addresses (like DMA regions), or static buffers, or stack allocated buffers.
    pub fn new(
        _lvgl: &Lvgl,
        display: T,
        // We don't need 'static. We could just create a generic lifetime, but let's keep things simple.
        draw_buffer: &'static mut [MaybeUninit<PixelColor>],
    ) -> Self {
        // We box the display to pin its address. This way, we can operate on it in the callback.
        let mut display = Box::new(display);

        let mut disp_draw_buf = unsafe {
            let mut disp_draw_buf = MaybeUninit::uninit();
            lvgl_sys::lv_disp_draw_buf_init(
                disp_draw_buf.as_mut_ptr(),
                draw_buffer.as_mut_ptr() as *mut cty::c_void,
                ptr::null_mut(),
                draw_buffer.len() as u32,
            );
            Box::new(disp_draw_buf.assume_init())
        };

        let mut disp_drv = unsafe {
            let mut disp_drv = MaybeUninit::uninit();
            lvgl_sys::lv_disp_drv_init(disp_drv.as_mut_ptr());
            let mut disp_drv = Box::new(disp_drv.assume_init());
            disp_drv.draw_buf = disp_draw_buf.as_mut();
            disp_drv.hor_res = display.size().width as lvgl_sys::lv_coord_t;
            disp_drv.ver_res = display.size().height as lvgl_sys::lv_coord_t;
            disp_drv.flush_cb = Some(Self::display_flush_cb);
            disp_drv.user_data = mem::transmute(display.as_mut());
            disp_drv
        };

        let disp = unsafe {
            lvgl_sys::lv_disp_drv_register(disp_drv.as_mut())
                .as_mut()
                .unwrap()
        };

        // If we wanted to cleanup resources on drop, these would have to be freed.
        Box::into_raw(disp_draw_buf);
        Box::into_raw(disp_drv);

        Self { disp, display }
    }

    unsafe extern "C" fn display_flush_cb(
        disp_drv: *mut lvgl_sys::lv_disp_drv_t,
        area: *const lvgl_sys::lv_area_t,
        color_p: *mut lvgl_sys::lv_color_t,
    ) {
        // In the `std` world we would make sure to capture panics here and make them not escape across
        // the FFI boundary. Since this library is focused on embedded platforms, we don't
        // have an standard unwinding mechanism to rely upon.
        let disp_drv = disp_drv.as_mut().unwrap();
        let display_ptr: *mut T = mem::transmute(disp_drv.user_data);
        let display = display_ptr.as_mut().unwrap();

        let area = Rectangle::with_corners(
            ((*area).x1 as i32, (*area).y1 as i32).into(),
            ((*area).x2 as i32, (*area).y2 as i32).into(),
        );

        let num_pixels = (area.size.width * area.size.height) as usize;
        let colors = core::slice::from_raw_parts(color_p as *const PixelColor, num_pixels);
        let colors = colors.iter().cloned();

        // Ignore errors
        let _ = display.fill_contiguous(&area, colors);

        // Indicate to LVGL that we are ready with the flushing
        // Note that we could do something async if we were to use something like DMA and two buffers.
        lvgl_sys::lv_disp_flush_ready(disp_drv);
    }
}

impl<T> Display<T> {
    pub fn load_screen<S>(&mut self, screen: &mut Screen<S>) {
        unsafe {
            lvgl_sys::lv_disp_load_scr(&mut *screen.raw);
        }
    }
}

impl<T> Deref for Display<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.display.deref()
    }
}

impl<T> DerefMut for Display<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.display.deref_mut()
    }
}

impl<T> Drop for Display<T> {
    fn drop(&mut self) {
        panic!("Display can't be dropped");
    }
}
