use cstr_core::CString;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use lvgl::core::Display;
use lvgl::core::Event;
use lvgl::core::InputDevice;
use lvgl::core::InputDeviceState;
use lvgl::core::Lvgl;
use lvgl::core::ObjExt;
use lvgl::core::Screen;
use lvgl::core::TouchPad;
use lvgl::style::Style;
use lvgl::{self, style, style::Align, style::Flag, style::GridAlign};
use std::time::Instant;
use std::{thread, time::Duration};

use std::mem;
const LVGL_BUFFER_LEN: usize = 76800;

use core::mem::MaybeUninit;

use lvgl::widgets::{Btn, Label};

pub struct BtnTest {
    style: Style,
    col_dsc: Box<[i16; 4]>,
    row_dsc: Box<[i16; 5]>,
    btn_0_1mm: Btn<BtnTest>,
    btn_1mm: Btn<BtnTest>,
    btn_10mm: Btn<BtnTest>,
    btn_up: Btn<BtnTest>,
    btn_home: Btn<BtnTest>,
    btn_down: Btn<BtnTest>,
    current_pos: Label<BtnTest>,
}

impl BtnTest {
    pub fn new(screen: &mut Screen<Self>) -> Self {
        let mut style = Style::new();
        style.set_pad_all(10);

        screen.add_style(&mut style, 0);

        let mut col_dsc = Box::new([
            style::grid_free(1),
            style::grid_free(1),
            style::grid_free(1),
            style::grid_last(),
        ]);
        let mut row_dsc = Box::new([
            style::grid_free(1),
            style::grid_free(1),
            style::grid_free(1),
            style::grid_free(1),
            style::grid_last(),
        ]);

        screen.set_grid_dsc_array(col_dsc.as_mut_ptr(), row_dsc.as_mut_ptr());

        let btn_0_1mm = Btn::new(screen).apply(|obj| {
            obj.on_event(Event::Clicked, |context| {
                println!("Clicked on: 0.1mm");
            })
            .set_grid_cell(GridAlign::Stretch, 0, 1, GridAlign::Stretch, 0, 1);

            let mut btn_lbl = Label::new(obj);
            btn_lbl.set_text(CString::new("0.1mm").unwrap().as_c_str());
            btn_lbl.align_to(obj, Align::Center, 0, 0);
        });

        let btn_1mm = Btn::new(screen).apply(|obj| {
            obj.on_event(Event::Clicked, |context| {
                println!("Clicked on: 1mm");
            })
            .set_grid_cell(GridAlign::Stretch, 1, 1, GridAlign::Stretch, 0, 1);

            let mut btn_lbl = Label::new(obj);
            btn_lbl.set_text(CString::new("1mm").unwrap().as_c_str());
            btn_lbl.align_to(obj, Align::Center, 0, 0);
        });

        let btn_10mm = Btn::new(screen).apply(|obj| {
            obj.on_event(Event::Clicked, |context| {
                println!("Clicked on: 10mm");
            })
            .set_grid_cell(GridAlign::Stretch, 2, 1, GridAlign::Stretch, 0, 1);

            let mut btn_lbl = Label::new(obj);
            btn_lbl.set_text(CString::new("10mm").unwrap().as_c_str());
            btn_lbl.align_to(obj, Align::Center, 0, 0);
        });

        let btn_up = Btn::new(screen).apply(|obj| {
            obj.on_event(Event::Clicked, |context| {
                println!("Clicked on: up");
            })
            .set_grid_cell(GridAlign::Stretch, 0, 1, GridAlign::Stretch, 1, 1);

            let mut btn_lbl = Label::new(obj);
            btn_lbl.set_text(CString::new("UP").unwrap().as_c_str());
            btn_lbl.align_to(obj, Align::Center, 0, 0);
        });

        let btn_home = Btn::new(screen).apply(|obj| {
            obj.on_event(Event::Clicked, |context| {
                println!("Clicked on: home");
            })
            .set_grid_cell(GridAlign::Stretch, 1, 1, GridAlign::Stretch, 1, 1);

            let mut btn_lbl = Label::new(obj);
            btn_lbl.set_text(CString::new("HOME").unwrap().as_c_str());
            btn_lbl.align_to(obj, Align::Center, 0, 0);
        });

        let btn_down = Btn::new(screen).apply(|obj| {
            obj.on_event(Event::Clicked, |context| {
                println!("Clicked on: down");
            })
            .set_grid_cell(GridAlign::Stretch, 2, 1, GridAlign::Stretch, 1, 1);

            let mut btn_lbl = Label::new(obj);
            btn_lbl.set_text(CString::new("DOWN").unwrap().as_c_str());
            btn_lbl.align_to(obj, Align::Center, 0, 0);
        });

        let btn_stop = Btn::new(screen).apply(|obj| {
            obj.on_event(Event::Clicked, |context| {
                println!("Clicked on: stop");
            })
            .set_grid_cell(GridAlign::Stretch, 0, 3, GridAlign::Stretch, 2, 1);

            let mut btn_lbl = Label::new(obj);
            btn_lbl.set_text(CString::new("STOP").unwrap().as_c_str());
            btn_lbl.align_to(obj, Align::Center, 0, 0);
        });

        let mut current_pos = Label::new(screen).apply(|obj| {
            obj.set_text(CString::new("0.0").unwrap().as_c_str());
            obj.set_grid_cell(GridAlign::Center, 0, 3, GridAlign::Center, 3, 1);
        });

        Self {
            style,
            col_dsc,
            row_dsc,
            btn_0_1mm,
            btn_1mm,
            btn_10mm,
            btn_up,
            btn_home,
            btn_down,
            current_pos,
        }
    }
    pub fn refresh(&mut self) {}
}

fn main() {
    let mut displays: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));

    let output_settings = OutputSettingsBuilder::new().scale(2).build();
    let mut window = Window::new("Bar Example", &output_settings);

    let mut lvgl = Lvgl::new();

    //lvgl.register_logger(|s| println!(s));

    static mut DRAW_BUFFER: [MaybeUninit<Rgb565>; LVGL_BUFFER_LEN] =
        [MaybeUninit::<Rgb565>::uninit(); LVGL_BUFFER_LEN];
    let mut displayd = Display::new(&lvgl, displays, unsafe { &mut DRAW_BUFFER });

    pub fn new_screen<D, C>(
        display: &Display<D>,
        init_f: impl FnOnce(&mut Screen<C>) -> C,
    ) -> Screen<C> {
        let mut screen = Screen::<C>::new(display);
        let context = init_f(&mut screen);
        screen.context().replace(context);
        screen
    }

    pub fn idle_task(
        mut lvgl: Lvgl,
        mut display: Display<SimulatorDisplay<Rgb565>>,
        mut window: Window,
    ) {
        let mut ui = new_screen(&display, |screen| BtnTest::new(screen));

        display.load_screen(&mut ui);

        ui.context().as_mut().unwrap().refresh();

        let mut lvgl_input_device = InputDevice::<TouchPad>::new(&mut display);

        pub fn into_lvgl_event_press(p: &Point) -> TouchPad {
            let e = TouchPad::Pressed {
                x: p.x as i16,
                y: p.y as i16,
            };
            let mut d: lvgl_sys::lv_indev_data_t = Default::default();
            let dm = &mut d;
            InputDeviceState::populate_lv_indev_data(&e, dm);
            println!("Size {}", mem::size_of::<lvgl_sys::lv_indev_data_t>());
            println!("State {}", d.state);
            e
        }

        pub fn into_lvgl_event_rel(p: &Point) -> TouchPad {
            let e = TouchPad::Released;
            let mut d: lvgl_sys::lv_indev_data_t = Default::default();
            let dm = &mut d;
            InputDeviceState::populate_lv_indev_data(&e, dm);
            println!("State {}", d.state);
            e
        }

        'running: loop {
            lvgl.run_tasks();

            window.update(&display);

            for event in window.events() {
                match event {
                    SimulatorEvent::MouseButtonDown {
                        mouse_btn: _,
                        point,
                    } => {
                        // Send a event to the button directly
                        //lvgl.event_send(&mut button, Event::Clicked);
                        *lvgl_input_device.state() = into_lvgl_event_press(&point);
                        println!("ClickD {} {}", point.x, point.y);
                    }
                    SimulatorEvent::MouseButtonUp {
                        mouse_btn: _,
                        point,
                    } => {
                        // Send a event to the button directly
                        //lvgl.event_send(&mut button, Event::Clicked);
                        *lvgl_input_device.state() = into_lvgl_event_rel(&point);
                        println!("ClickU {} {}", point.x, point.y);
                    }
                    SimulatorEvent::Quit => break 'running,
                    _ => {}
                }
            }

            thread::sleep(Duration::from_millis(1));
            lvgl.ticks().inc(1000);

            //ticks.inc(loop_started.elapsed().as_millis() as u32);
        }
    }

    idle_task(lvgl, displayd, window);

    // Create the button

    //let mut btn_state = false;
    //button.on_event(|mut btn, event| {
    //    if let lvgl::Event::Clicked = event {
    //        if btn_state {
    //            let nt = CString::new("Click me!").unwrap();
    //            btn_lbl.set_text(nt.as_c_str());
    //        } else {
    //            let nt = CString::new("Clicked!").unwrap();
    //            btn_lbl.set_text(nt.as_c_str());
    //        }
    //        btn_state = !btn_state;
    //        println!("Clicked! Inner..");
    //        btn.toggle().unwrap();
    //    }
    //});

    //    let ticks = lvgl.ticks();
    //
    //    let mut loop_started = Instant::now();
    //    'running: loop {
    //        lvgl.run_tasks();
    //        window.update(&displayd);
    //        //ui.context().as_mut().unwrap().refresh();
    //
    //        for event in window.events() {
    //            match event {
    //                SimulatorEvent::MouseButtonDown {
    //                    mouse_btn: _,
    //                    point,
    //                } => {
    //                    println!("Clicked on: {:?}", point);
    //                    // Send a event to the button directly
    //                    //lvgl.event_send(&mut button, Event::Clicked);
    //                }
    //                SimulatorEvent::Quit => break 'running,
    //                _ => {}
    //            }
    //        }
    //
    //        //ticks.inc(loop_started.elapsed().as_millis() as u32);
    //        loop_started = Instant::now();
    //    }
}
