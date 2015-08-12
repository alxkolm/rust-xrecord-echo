use x11::xlib;
use std::ptr::{
  null,
  null_mut,
};
use x11wrapper::window::{Window};

pub struct Display {
    pub display: *mut xlib::Display,
}

impl Display {
    pub fn new() -> Display {
        Display {display: unsafe {
            let dpy = xlib::XOpenDisplay(null());
            if dpy == null_mut() {
                panic!("XOpenDisplay() failed!");
            }
            dpy
        }}
    }
    pub fn get_input_focus(&self) -> Window{
        let current_window: *mut xlib::Window = &mut 0;
        let revert_to_return: *mut i32 = &mut 0;
        unsafe{xlib::XGetInputFocus(self.display, current_window, revert_to_return)};
        Window {id: unsafe{*current_window as usize}, display: self.display}
    }

    pub fn window(&self, xid: usize) -> Window {
        Window {id: xid, display: self.display}
    }
}