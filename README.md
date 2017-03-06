`xrecord-echo` echo XRecord events by nanomsg library. Any can subsribe to it.

Event message contains info about window and process where it triggered. Message looks like that:

```json
{
  "event_type": "KeyEvent",
  "xserver_time": 12255472,
  "timestamp": 1441459157,
  "wm_name": "alxkolm/rust-xrecord-echo - Google Chrome",
  "wm_class": "Google-chrome-stable",
  "pid": 5077,
  "proc_name": "chrome",
  "proc_cmd": "/opt/google/chrome/chrome",
  "code": 11
}
```

## Install

### Prerequisites

- Rust compiler
- nanomsg (tested on >=0.5)

To install all of prerequisites and build *xrecord-echo* run:

    make

To just build *xrecord-echo* use *cargo*:
    
    cargo build --release

## Run

To run *xrecord-echo* use `start` script. It automaticaly respawn process when it crush.

But I recommend use supervisor like [*runit*](http://smarden.org/runit/) to manage process (run on system startup and restart on crash).

*xrecord-echo* emit messages as publisher on port 1234.

To check that this work use nanomsg tools to subscribe and output to terminal:
    
    nn_sub -l1234 -A

## Setup systemd

  cp xrecord-echo.service ~/.config/systemd/user
  systemctl --user enable xrecord-echo.service
  systemctl --user start xrecord-echo.service

## Todo

- [ ] Customize *nanomsg* port by command line parameters
- [ ] Implement XRecord *error callback*
