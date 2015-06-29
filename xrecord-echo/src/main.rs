extern crate libc;
extern crate x11;
extern crate nanomsg;
extern crate time;
#[macro_use]
extern crate lazy_static;


use x11wrapper::{Display, Window, WindowTree};
use x11::xlib;
use x11::xrecord;
use std::ffi::CString;
use libc::{c_int};
use nanomsg::{Socket, Protocol};
use std::io::{Write, Error};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::Read;

mod x11wrapper;

struct XRecordDatum {
    xtype: u8,
    code: u8,
    unknown1: u8,
    unknown2: u8
}

struct ServiceData {
    socket: Socket,
    procs: HashMap<u32, Process>
}

#[derive(Clone)]
struct Process {
    name: String,
    cmdline: String
}

static mut event_count:u64 = 0;
static mut start_local_time: i64 = 0;
static mut start_server_time: i64 = 0;

static mut display_control: Display = Display {display: 0 as *mut xlib::Display};
static mut display_data: Display = Display {display: 0 as *mut xlib::Display};

fn main() {
    unsafe{
        record_bootstrap();
    }
}

unsafe fn record_bootstrap () {
    let procs: HashMap<u32, Process> = HashMap::new();

    display_control = Display::new();
    display_data = Display::new();

    xlib::XSynchronize(display_control.display, 1);

    let extension_name = CString::new("RECORD").unwrap();
        
    let extension = xlib::XInitExtension(
        display_control.display,
        extension_name.as_ptr());
    if extension.is_null() {
        panic!("Error init X Record Extension");
    }

    // Get version
    let mut version_major: c_int = 0;
    let mut version_minor: c_int = 0;
    xrecord::XRecordQueryVersion(
        display_control.display,
        &mut version_major,
        &mut version_minor
    );
    println!(
        "RECORD extension version {}.{}",
        version_major,
        version_minor
    );

    // Prepare record range
    let mut record_range: xrecord::XRecordRange = *xrecord::XRecordAllocRange();
    record_range.device_events.first = xlib::KeyPress as u8;
    record_range.device_events.last = xlib::MotionNotify as u8;
    record_range.delivered_events.first = xlib::EnterNotify as u8;
    record_range.delivered_events.last = xlib::EnterNotify as u8;

    // Create context
    let context = xrecord::XRecordCreateContext(
        display_control.display,
        0,
        &mut xrecord::XRecordAllClients,
        1,
        std::mem::transmute(&mut &mut record_range),
        1
    );

    if context == 0 {
        panic!("Fail create Record context\n");
    }

    // let mut windowSniffer = WindowSniffer::new();
    

    let mut socket = Socket::new(Protocol::Pub).unwrap();
    let endpoint = socket.bind("tcp://127.0.0.1:1234");

    let mut service = ServiceData {socket: socket, procs: procs};

    // Run
    let res = xrecord::XRecordEnableContext(
        display_data.display,
        context,
        Some(record_callback),
        std::mem::transmute(&mut service));

    if res == 0 {
        panic!("Cound not enable the Record context!\n");
    }
    xrecord::XRecordFreeContext(display_data.display, context);
}

unsafe extern "C" fn record_callback(pointer:*mut i8, raw_data: *mut xrecord::XRecordInterceptData) {
    let service: &mut ServiceData = std::mem::transmute(pointer);
        
    let data = &*raw_data;
    
    match data.category {
        xrecord::XRecordStartOfData => {
            start_local_time = time::get_time().sec;
            start_server_time = data.server_time as i64;
        },
        xrecord::XRecordFromServer => {},
        _ => {}
    }
    if data.category != xrecord::XRecordFromServer {
        return;
    }
    event_count += 1;
    let xdatum = &*(data.data as *mut XRecordDatum);

    // Detect wm_name
    
    let window = get_current_window();
    // (*sniffer).processEvent(window);
    // if window.is_none() {
    //  return;
    // }
     
    // Count events
    let event = match xdatum.xtype as i32 {
        xlib::KeyPress     => Some(UserEvent::KeyEvent{time: data.server_time as usize, keycode: 1}),
        xlib::ButtonPress  => Some(UserEvent::ClickEvent{time: data.server_time as usize, buttoncode: 1}),
        xlib::MotionNotify => Some(UserEvent::MotionEvent{time: data.server_time as usize}),
        xlib::EnterNotify  => Some(UserEvent::EnterEvent{time: data.server_time as usize}),
        _                  => None
    };

    match event {
        Some(e) => {
            send_event(window, e, service);
            // (*sniffer).processEvent(window, e);
        },
        _ => {}
    }
    
    xrecord::XRecordFreeData(raw_data);
}

fn get_current_window() -> Window {
    let mut current_window = unsafe {display_control.get_input_focus()};
    let mut parent_window: Option<Window> = None;
    let mut wm_name_str: Option<String> = None;
    
    let mut i = 0u32;
    while i < 10 {
        if current_window.id == 0  || current_window.id == 1 {
            break;
        }
        
        wm_name_str = current_window.get_wm_name();
        if wm_name_str.is_none() || wm_name_str.clone().unwrap() == "FocusProxy".to_string() {
            // If not found or wmname is "FocusProxy" dig up to tree
            let tree = current_window.get_tree();
            parent_window = match tree {
                Some(WindowTree{parent, children: _}) => {
                    Some(parent)
                },
                _ => None
            }
        } else {
            // Found window with adequate WM_NAME.
            // Exit from while loop.
            break;
        }
                    
        current_window = match parent_window {
            Some(win) => win,
            _ => current_window
        };
        
        i += 1;
    }
    current_window
    // selftop::Window {
    //     wm_name: current_window.get_wm_name(),
    //     class: current_window.get_class(),
    //     pid: current_window.get_pid(),
    // }
}


pub enum UserEvent {
    MotionEvent{time: usize},
    EnterEvent{time: usize},
    KeyEvent{keycode: u8, time: usize},
    ClickEvent{buttoncode: u8, time: usize}
}

fn send_event(window: Window, event: UserEvent, service: &mut ServiceData) -> Result<usize, Error> {
    let ref mut socket = service.socket;
    let ref mut procs = service.procs;

    let event_type = match event {
        UserEvent::MotionEvent{..} => "MotionEvent".to_string(),
        UserEvent::EnterEvent{..}  => "EnterEvent".to_string(),
        UserEvent::KeyEvent{..}    => "KeyEvent".to_string(),
        UserEvent::ClickEvent{..}  => "ClickEvent".to_string() 
    };

    let server_time = match event {
        UserEvent::MotionEvent{time, ..} => time,
        UserEvent::EnterEvent{time, ..}  => time,
        UserEvent::KeyEvent{time, ..}    => time,
        UserEvent::ClickEvent{time, ..}  => time
    };

    let wm_name = match window.get_wm_name() {
        Some(title) => title.to_string(),
        _           => "".to_string()
    };

    let class = match window.get_class() {
        Some(classes) => classes[classes.len()-1].to_string(),
        _             => "".to_string()
    };

    let pid = match window.get_pid() {
        Some(pid) => pid,
        _             => 0
    };

    if !procs.contains_key(&pid) {
        let process = match get_proc_name(pid) {
            Some(p) =>  p,
            _          => Process{name: "".to_string(), cmdline: "".to_string()}
        };
        procs.insert(pid.clone(), process.clone());
    };

    let process: Process = match procs.get(&pid) {
        Some(ref p) => {
            (*p).clone()
        },
        _ => Process{name: "".to_string(), cmdline: "".to_string()}
    };

    let mut timestamp:i64 = 0;
    unsafe{
        timestamp = start_local_time + ((((server_time as f64) - start_server_time as f64))/1000.0).floor() as i64;
    }
    
    let message = format!(
        "xrecord\n{event_type}\n{time}\n{wm_name}\n{class}\n{timestamp}\n{pid}\n{proc_name}\n{proc_cmd}",
        event_type = event_type,
        time       = server_time,
        wm_name    = wm_name,
        class      = class,
        timestamp  = timestamp,
        pid        = pid,
        proc_name  = process.name,
        proc_cmd   = process.cmdline
    );

    socket.write(message.as_bytes())
}

fn get_proc_name(pid: u32) -> Option<Process> {
    if pid == 0 {
        return None;
    }

    let mut f = File::open("/proc/".to_string() + &(pid.to_string()) + &("/comm".to_string()));
    let name = match f {
        Ok(mut file) => {
            let mut name = String::new();
            file.read_to_string(&mut name);
            name
        },
        _ => "".to_string()
    };

    let mut f2 = File::open("/proc/".to_string() + &(pid.to_string()) + &("/cmdline".to_string()));
    let cmd = match f2 {
        Ok(mut file) => {
            let mut name = String::new();
            file.read_to_string(&mut name);
            name
        },
        _ => "".to_string()
    };

    Some(Process {
        name: name.trim().to_string(),
        cmdline: cmd.trim().to_string()
    })
}