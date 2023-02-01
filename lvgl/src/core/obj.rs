use lvgl_sys::{lv_coord_t, lv_obj_t};

use core::{
    ops::{Deref, DerefMut},
    ptr,
};

use crate::{
    core::event::add_event_cb,
    core::Event,
    style::{Align, Flag, GridAlign, State},
};

use crate::style::Style;
use cty::uint8_t;

/// Base LVGL object. C is the application context that we provide to the
/// callbacks The lifetime of the object depends on the lifetime of its parent.
/// (in lvgl, deleting an object deletes all its children).
/// We have not implemented lifetimes correctly at this point.
pub struct Obj<C> {
    // pub so that the user can use lvgl_sys functions directly
    pub raw: &'static mut lv_obj_t,
    // We want a stable pointer.
    pub(crate) context: ptr::NonNull<Option<C>>, // Should be a refcell? Or a &mut? Lost patience trying to make things work.
}

unsafe impl<C> Send for Obj<C> {}

impl<C> Obj<C> {
    pub fn from_raw(raw: &'static mut lv_obj_t, context: ptr::NonNull<Option<C>>) -> Self {
        Self { raw, context }
    }
}

pub trait ObjExt<C: 'static>: Deref<Target = Obj<C>> + DerefMut + Sized {
    fn context(&mut self) -> &mut Option<C> {
        unsafe { self.context.as_mut() }
    }

    fn apply(mut self, f: impl FnOnce(&mut Self)) -> Self {
        // We don't care about the return value. It's typically &mut self
        f(&mut self);
        self
    }

    /// Register an event callback, for a specific event
    fn on_event(&mut self, event: Event, mut f: impl FnMut(&mut C) + 'static) -> &mut Self {
        let mut context = self.context;
        add_event_cb(self.raw, Some(event), move |_e, _current_target, _child| {
            let context = unsafe { context.as_mut().as_mut() };
            let context = context.expect("screen.context() must be set");
            f(context)
        });
        self
    }

    /// Register an event callback, receiving all events.
    fn on_any_event(&mut self, mut f: impl FnMut(&mut C, Event) + 'static) -> &mut Self {
        let mut context = self.context;
        add_event_cb(self.raw, None, move |e, _current_target, _child| {
            let context = unsafe { context.as_mut().as_mut() };
            let context = context.expect("screen.context() must be set");
            f(context, e)
        });
        self
    }

    fn align_to(
        &mut self,
        base: &impl ObjExt<C>,
        align: Align,
        x_mod: lv_coord_t,
        y_mod: lv_coord_t,
    ) -> &mut Self {
        unsafe { lvgl_sys::lv_obj_align_to(&mut *self.raw, base.raw, align.into(), x_mod, y_mod) };
        self
    }

    fn set_pos(&mut self, x: lv_coord_t, y: lv_coord_t) -> &mut Self {
        unsafe { lvgl_sys::lv_obj_set_pos(&mut *self.raw, x, y) };
        self
    }

    fn set_size(&mut self, w: lv_coord_t, h: lv_coord_t) -> &mut Self {
        unsafe { lvgl_sys::lv_obj_set_size(&mut *self.raw, w, h) };
        self
    }

    fn set_width(&mut self, w: lv_coord_t) -> &mut Self {
        unsafe { lvgl_sys::lv_obj_set_width(&mut *self.raw, w) };
        self
    }

    fn set_height(&mut self, h: lv_coord_t) -> &mut Self {
        unsafe { lvgl_sys::lv_obj_set_height(&mut *self.raw, h) };
        self
    }

    fn add_flag(&mut self, flag: Flag) -> &mut Self {
        unsafe { lvgl_sys::lv_obj_add_flag(&mut *self.raw, flag.bits()) };
        self
    }

    fn clear_flag(&mut self, flag: Flag) -> &mut Self {
        unsafe { lvgl_sys::lv_obj_clear_flag(&mut *self.raw, flag.bits()) };
        self
    }

    fn has_flag(&self, flag: Flag) -> bool {
        unsafe { lvgl_sys::lv_obj_has_flag(&*self.raw, flag.bits()) }
    }

    fn add_state(&mut self, state: State) -> &mut Self {
        unsafe { lvgl_sys::lv_obj_add_state(&mut *self.raw, state.bits()) };
        self
    }

    fn clear_state(&mut self, state: State) -> &mut Self {
        unsafe { lvgl_sys::lv_obj_clear_state(&mut *self.raw, state.bits()) };
        self
    }

    fn get_state(&self) -> State {
        let state = unsafe { lvgl_sys::lv_obj_get_state(&*self.raw) };
        State::from_bits(state).unwrap()
    }

    fn has_state(&self, state: State) -> bool {
        unsafe { lvgl_sys::lv_obj_has_state(&*self.raw, state.bits()) }
    }

    fn set_grid_dsc_array(
        &mut self,
        col_dsc: *mut lvgl_sys::lv_coord_t,
        row_dsc: *mut lvgl_sys::lv_coord_t,
    ) {
        unsafe { lvgl_sys::lv_obj_set_grid_dsc_array(&mut *self.raw, col_dsc, row_dsc) };
    }

    fn set_grid_cell(
        &mut self,
        column_align: GridAlign,
        col_pos: uint8_t,
        col_span: uint8_t,
        row_align: GridAlign,
        row_pos: uint8_t,
        row_span: uint8_t,
    ) {
        unsafe {
            lvgl_sys::lv_obj_set_grid_cell(
                &mut *self.raw,
                column_align as u8,
                col_pos,
                col_span,
                row_align as u8,
                row_pos,
                row_span,
            )
        };
    }

    fn add_style(&mut self, style: &mut Style, selector: u32) {
        unsafe { lvgl_sys::lv_obj_add_style(&mut *self.raw, &mut *style.raw, selector) }
    }
}

impl<C: 'static, T: Deref<Target = Obj<C>> + DerefMut + Sized> ObjExt<C> for T {}

macro_rules! define_object {
    ($item:ident) => {
        pub struct $item<C> {
            pub(crate) obj: Obj<C>,
        }

        impl<C> core::ops::Deref for $item<C> {
            type Target = Obj<C>;

            fn deref(&self) -> &Self::Target {
                &self.obj
            }
        }

        impl<C> core::ops::DerefMut for $item<C> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.obj
            }
        }
    };
}
