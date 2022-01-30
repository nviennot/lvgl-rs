//use crate::macros::native_enum;

use alloc::boxed::Box;
use crate::{Obj, Widget};

use core::convert::TryInto;
use core::ptr::NonNull;

native_enum! {
    lvgl_sys::lv_event_code_t,
    /// Events are triggered in LVGL when something happens which might be interesting to
    /// the user, e.g. if an object:
    ///  - is clicked
    ///  - is dragged
    ///  - its value has changed, etc.
    ///
    /// All objects (such as Buttons/Labels/Sliders etc.) receive these generic events
    /// regardless of their type.
    pub enum Event {
        /** Input device events*/
        /// The object has been pressed
        Pressed = lvgl_sys::lv_event_code_t_LV_EVENT_PRESSED,
        /// The object is being pressed (called continuously while pressing)
        Pressing = lvgl_sys::lv_event_code_t_LV_EVENT_PRESSING,
        /// The object is still being pressed but slid cursor/finger off of the object
        PressLost = lvgl_sys::lv_event_code_t_LV_EVENT_PRESS_LOST,
        /// The object was pressed for a short period of time, then released it. Not called if scrolled.
        ShortClicked = lvgl_sys::lv_event_code_t_LV_EVENT_SHORT_CLICKED,
        /// Object has been pressed for at least `long_press_time`.  Not called if scrolled.
        LongPressed = lvgl_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED,
        /// Called after `long_press_time` in every `long_press_repeat_time` ms.  Not called if scrolled.
        LongPressedRepeat = lvgl_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED_REPEAT,
        /// Called on release if not scrolled (regardless to long press)
        Clicked = lvgl_sys::lv_event_code_t_LV_EVENT_CLICKED,
        /// Called in every cases when the object has been released
        Released = lvgl_sys::lv_event_code_t_LV_EVENT_RELEASED,
        /// Scrolling begins
        ScrollBegin = lvgl_sys::lv_event_code_t_LV_EVENT_SCROLL_BEGIN,
        /// Scrolling ends
        ScrollEnd = lvgl_sys::lv_event_code_t_LV_EVENT_SCROLL_END,
        /// Scrolling
        Scroll = lvgl_sys::lv_event_code_t_LV_EVENT_SCROLL,
        /// A gesture is detected. Get the gesture with `lv_indev_get_gesture_dir(lv_indev_get_act());`
        Gesture = lvgl_sys::lv_event_code_t_LV_EVENT_GESTURE,
        /// A key is sent to the object. Get the key with `lv_indev_get_key(lv_indev_get_act());`
        Key = lvgl_sys::lv_event_code_t_LV_EVENT_KEY,
        /// The object is focused
        Focused = lvgl_sys::lv_event_code_t_LV_EVENT_FOCUSED,
        /// The object is defocused
        Defocused = lvgl_sys::lv_event_code_t_LV_EVENT_DEFOCUSED,
        /// The object is defocused but still selected
        Leave = lvgl_sys::lv_event_code_t_LV_EVENT_LEAVE,
        /// Perform advanced hit-testing
        HitTest = lvgl_sys::lv_event_code_t_LV_EVENT_HIT_TEST,

        /** Drawing events*/
        /// Check if the object fully covers an area. The event parameter is `lv_cover_check_info_t *`.
        CoverCheck = lvgl_sys::lv_event_code_t_LV_EVENT_COVER_CHECK,
        /// Get the required extra draw area around the object (e.g. for shadow). The event parameter is `lv_coord_t *` to store the size.
        RefrExtDrawSize = lvgl_sys::lv_event_code_t_LV_EVENT_REFR_EXT_DRAW_SIZE,
        /// Starting the main drawing phase
        DrawMainBegin = lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_MAIN_BEGIN,
        /// Perform the main drawing
        DrawMain = lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_MAIN,
        /// Finishing the main drawing phase
        DrawMainEnd = lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_MAIN_END,
        /// Starting the post draw phase (when all children are drawn)
        DrawPostBegin = lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_POST_BEGIN,
        /// Perform the post draw phase (when all children are drawn)
        DrawPost = lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_POST,
        /// Finishing the post draw phase (when all children are drawn)
        DrawPostEnd = lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_POST_END,
        /// Starting to draw a part. The event parameter is `lv_obj_draw_dsc_t *`.
        DrawPartBegin = lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_PART_BEGIN,
        /// Finishing to draw a part. The event parameter is `lv_obj_draw_dsc_t *`.
        DrawPartEnd = lvgl_sys::lv_event_code_t_LV_EVENT_DRAW_PART_END,

        /** Special events*/
        /// The object's value has changed (i.e. slider moved)
        ValueChanged = lvgl_sys::lv_event_code_t_LV_EVENT_VALUE_CHANGED,
        /// A text is inserted to the object. The event data is `char *` being inserted.
        Insert = lvgl_sys::lv_event_code_t_LV_EVENT_INSERT,
        /// Notify the object to refresh something on it (for the user)
        Refresh = lvgl_sys::lv_event_code_t_LV_EVENT_REFRESH,
        /// A process has finished
        Ready = lvgl_sys::lv_event_code_t_LV_EVENT_READY,
        /// A process has been cancelled
        Cancel = lvgl_sys::lv_event_code_t_LV_EVENT_CANCEL,

        /** Other events*/
        /// Object is being deleted
        Delete = lvgl_sys::lv_event_code_t_LV_EVENT_DELETE,
        /// Child was removed, added, or its size, position were changed
        ChildChanged = lvgl_sys::lv_event_code_t_LV_EVENT_CHILD_CHANGED,
        /// Child was created, always bubbles up to all parents
        ChildCreated = lvgl_sys::lv_event_code_t_LV_EVENT_CHILD_CREATED,
        /// Child was deleted, always bubbles up to all parents
        ChildDeleted = lvgl_sys::lv_event_code_t_LV_EVENT_CHILD_DELETED,
        /// A screen unload started, fired immediately when scr_load is called
        ScreenUnloadStart = lvgl_sys::lv_event_code_t_LV_EVENT_SCREEN_UNLOAD_START,
        /// A screen load started, fired when the screen change delay is expired
        ScreenLoadStart = lvgl_sys::lv_event_code_t_LV_EVENT_SCREEN_LOAD_START,
        /// A screen was loaded
        ScreenLoaded = lvgl_sys::lv_event_code_t_LV_EVENT_SCREEN_LOADED,
        /// A screen was unloaded
        ScreenUnloaded = lvgl_sys::lv_event_code_t_LV_EVENT_SCREEN_UNLOADED,
        /// Object coordinates/size have changed
        SizeChanged = lvgl_sys::lv_event_code_t_LV_EVENT_SIZE_CHANGED,
        /// Object's style has changed
        StyleChanged = lvgl_sys::lv_event_code_t_LV_EVENT_STYLE_CHANGED,
        /// The children position has changed due to a layout recalculation
        LayoutChanged = lvgl_sys::lv_event_code_t_LV_EVENT_LAYOUT_CHANGED,
        /// Get the internal size of a widget
        GetSelfSize = lvgl_sys::lv_event_code_t_LV_EVENT_GET_SELF_SIZE,
    }
}


/// FIXME This is not ideal. This will get duplicated for each callback, and eat all our ROM

pub(crate) fn add_event_cb<T, F>(target: &mut T, cb: F, event: Option<Event>)
where
    T: Widget + Sized,
    F: FnMut(T, Event, Option<Obj>),
{
    let user_closure = Box::new(cb);
    let user_data = Box::into_raw(user_closure) as *mut cty::c_void;
    let event = event.map(|e| e.into()).unwrap_or(lvgl_sys::lv_event_code_t_LV_EVENT_ALL);

    unsafe {
        lvgl_sys::lv_obj_add_event_cb(
            target.raw().as_mut(),
            Some(event_callback::<T, F>),
            event,
            user_data,
        );
    }
}

unsafe extern "C" fn event_callback<T, F>(
    event: *mut lvgl_sys::lv_event_t,
) where
    T: Widget + Sized,
    F: FnMut(T, Event, Option<Obj>),
{
    // Seems a bit silly to use functions to access fields, but that's what the
    // libary example show.

    let event_code = lvgl_sys::lv_event_get_code(event);
    let current_target = lvgl_sys::lv_event_get_current_target(event);
    let target = lvgl_sys::lv_event_get_target(event);
    let user_data = lvgl_sys::lv_event_get_user_data(event);

    if let Ok(event_code) = event_code.try_into() {
        if let Some(current_target) = NonNull::new(current_target) {
            if let Some(target) = NonNull::new(target) {
                // current_target is always the object on which .on_event() was called.
                // (So it can be casted to T)
                // target can either be the same object, or a child object,
                // when LV_OBJ_FLAG_EVENT_BUBBLE is set on the child.
                let child = if current_target != target {
                    Some(Obj::from_raw(target))
                } else {
                    None
                };

                let current_target = T::from_raw(current_target);
                let user_closure = &mut *(user_data as *mut F);
                user_closure(current_target, event_code, child);
            }
        }
    }
}
