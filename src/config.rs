use libc::c_ulong;
use x11::xlib;
use std::ffi::CString;

// For convenience
pub const MODKEY1: u32 = xlib::Mod1Mask;
pub const MODKEY2: u32 = xlib::Mod4Mask;
pub const SHIFT: u32 = xlib::ShiftMask;

// Key combos. We add our bindings here for wm actions
pub const EXIT_KEY: KeyCmd<'static> = KeyCmd{ key: "F1", modifier: MODKEY2|SHIFT };
pub const TERM_KEY: KeyCmd<'static> = KeyCmd{ key: "t", modifier: MODKEY2|SHIFT };
pub const RUN_KEY: KeyCmd<'static> = KeyCmd{ key: "r", modifier: MODKEY2 };

pub const RUN: &'static str = "dmenu_run";
pub const TERMINAL: &'static str = "termite";

// Mouse commands. We add our binds here for wm actions
pub const MOUSE_MOVE: MouseCmd = MouseCmd{ button: 1, modifier: MODKEY2 };
pub const MOUSE_RAISE: MouseCmd = MouseCmd{ button: 1, modifier: 0 };
pub const MOUSE_RESIZE: MouseCmd = MouseCmd{ button: 3, modifier: MODKEY2 };

// Mouse focus behavior
pub const SLOPPYFOCUS: bool = false;

// Borders. NOTE Color format is "rgb:ff/ff/ff" failure to use this code format
// will result in a segfault.
pub const BORDER0: Border<'static> = Border{ size: 2, color: "rgb:a5/a5/a5" };
pub const BORDER1: Border<'static> = Border{ size: 2, color: "rgb:18/18/18" };
pub const BORDER2: Border<'static> = Border{ size: 2, color: "rgb:aa/ff/33" };
pub const BORDER3: Border<'static> = Border{ size: 2, color: "rgb:00/bb/aa" };

pub const FBORDER0: Border<'static> = Border{ size: 2, color: "rgb:ff/00/00" };
pub const FBORDER1: Border<'static> = Border{ size: 2, color: "rgb:00/ff/00" };
pub const FBORDER2: Border<'static> = Border{ size: 2, color: "rgb:00/00/ff" };
pub const FBORDER3: Border<'static> = Border{ size: 2, color: "rgb:ff/ff/ff" };

pub const NUM_UNFOCUSED_BORDERS: usize = 4;
pub const UNFOCUSED_BORDERS: [Border<'static>; NUM_UNFOCUSED_BORDERS] =
            [ BORDER0, BORDER1, BORDER2, BORDER3 ];

pub const NUM_FOCUSED_BORDERS: usize = 4;
pub const FOCUS_BORDERS: [Border<'static>; NUM_FOCUSED_BORDERS] =
            [ FBORDER0, FBORDER1, FBORDER2, FBORDER3 ];



// Structs for configs
pub struct Border<'a> {
    pub size: i32,
    pub color: &'a str,
}

pub struct BorderInfo {
    focus_size: i32,
    unfocus_size: i32,
}

impl BorderInfo {
    pub fn new( focus: [Border;NUM_FOCUSED_BORDERS],
                unfocus: [Border;NUM_UNFOCUSED_BORDERS] ) -> BorderInfo {
        let focus_iter = focus.into_iter();
        let mut focus_size = 0;
        for border in focus_iter {
            focus_size += border.size;
        }

        let unfocus_iter = unfocus.into_iter();
        let mut unfocus_size = 0;
        for border in unfocus_iter {
            unfocus_size += border.size;
        }

        BorderInfo {
            focus_size: focus_size,
            unfocus_size: unfocus_size,
        }
    }

    pub fn get_focus_size( &self ) -> i32 {
        self.focus_size
    }
    pub fn get_unfocus_size( &self ) -> i32 {
        self.unfocus_size
    }
}


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
