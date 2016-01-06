use libc::c_ulong;
use x11::xlib;
use std::ffi::CString;

// For convenience
pub const MODKEY1: u32 = xlib::Mod1Mask;
pub const MODKEY2: u32 = xlib::Mod4Mask;
pub const SHIFT: u32 = xlib::ShiftMask;

// Key combos. We add our bindings here for wm actions
pub const EXIT_KEY: KeyCmd<'static> = KeyCmd{ key: "F1", modifier: MODKEY2|SHIFT };

// Mouse commands. We add our binds here for wm actions
pub const MOUSE_MOVE: MouseCmd = MouseCmd{ button: 1, modifier: MODKEY2 };
pub const MOUSE_RAISE: MouseCmd = MouseCmd{ button: 1, modifier: 0 };
pub const MOUSE_RESIZE: MouseCmd = MouseCmd{ button: 3, modifier: MODKEY2 };

#[derive(PartialEq)]
pub struct MouseCmd {
    pub button: u32,
    pub modifier: u32,
}

impl MouseCmd {
    pub fn new( button: u32, modifier: u32 ) -> MouseCmd {
        MouseCmd {
            button: button,
            modifier: modifier,
        }
    }
}

#[derive(PartialEq)]
pub struct KeyCmd<'a> {
    key: &'a str,
    modifier: u32,
}

impl<'a> KeyCmd<'a> {
    pub fn new( key: &'a str, modifier: u32 ) -> KeyCmd {
        KeyCmd {
            key: key,
            modifier: modifier,
        }
    }

    pub fn get_keysym( key: KeyCmd ) -> c_ulong {
        let key_string = CString::new( key.key ).unwrap();
        unsafe {
            xlib::XStringToKeysym( key_string.as_ptr() )
        }
    }

    pub fn get_modifier( key: KeyCmd ) -> u32 {
        key.modifier
    }

    pub fn get_key( key: KeyCmd ) -> CString {
        CString::new( key.key ).unwrap()
    }
}
