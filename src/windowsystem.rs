use libc::c_ulong;
use std::ffi::CString;
use std::ptr::{
  null,
  null_mut,
};

use x11::xlib;
use keys;
use keys::Key;

pub struct WindowSystem {
    display:    *mut xlib::Display,
    root:       xlib::Window,
}

impl WindowSystem {
    pub fn new() -> WindowSystem {
        unsafe {
            // Open display
            let display = xlib::XOpenDisplay(null());
            if display == null_mut() {
                panic!("Exiting: Cannot find display");
            }

            // Create window
            let screen = xlib::XDefaultScreenOfDisplay(display);
            let root = xlib::XRootWindowOfScreen(screen);

            WindowSystem {
                display: display,
                root: root,
            }
        }
    }

    pub fn on_init( &mut self ) {
        use x11::xlib::*;
        let mut wa = XSetWindowAttributes {
            background_pixmap: 0,
            background_pixel: 0,
            border_pixmap: 0,
            border_pixel: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: 0,
            do_not_propagate_mask: 0,
            override_redirect: 0,
            colormap: 0,
            cursor: 0,
            event_mask:
                        SubstructureRedirectMask|
                        SubstructureNotifyMask|
                        ButtonPressMask|
                        ButtonReleaseMask|
                        PointerMotionMask|
                        EnterWindowMask|
                        LeaveWindowMask|
                        StructureNotifyMask|
                        PropertyChangeMask,
        };

        unsafe {

            // Grab keys
            // Exit key behavior
            let kc_exit = xlib::XKeysymToKeycode( self.display, Key::get_keysym( keys::EXIT_KEY ) );
            xlib::XGrabKey( self.display, kc_exit as i32, Key::get_modifier( keys::EXIT_KEY ),
                self.root as c_ulong, 1, xlib::GrabModeAsync, xlib::GrabModeAsync );

            // Grab mouse
            xlib::XGrabButton( self.display, keys::MOUSE_MOVE.button, keys::MOUSE_MOVE.modifier,
                self.root, 1, xlib::ButtonPressMask as u32, xlib::GrabModeAsync, xlib::GrabModeAsync, 0, 0 );

            // Set up window events
            XChangeWindowAttributes( self.display, self.root, CWEventMask|CWCursor, &mut wa );
            XSelectInput( self.display, self.root, wa.event_mask );
        }
    }

    pub fn on_update( &mut self ) -> bool {
        let mut ev = xlib::XEvent { pad : [0; 24] };
        unsafe {
            xlib::XNextEvent( self.display, &mut ev );
        }
        let event_type = ev.get_type();

        match event_type {
            xlib::MotionNotify => {
            },

            xlib::ButtonPress => {
                let event = xlib::XButtonEvent::from(ev);
                unsafe {
                    self.on_button_press( &event );
                }
            },

            xlib::ButtonRelease => {
                let event = xlib::XButtonEvent::from(ev);
                unsafe {
                    self.on_button_release( &event );
                }
            },

            xlib::ClientMessage => {
                let event = xlib::XClientMessageEvent::from(ev);
                unsafe {
                    self.on_client_message( &event );
                }
            },

            xlib::DestroyNotify => {
                println!("Got destroy notify");
            },

            xlib::MapRequest => {
                let mut event = xlib::XMapRequestEvent::from(ev);
                unsafe {
                    self.on_map_request( &mut event );
                }
            },

//            xlib::CreateNotify => {
//                println!("Got create notify");
//            }

            xlib::KeyPress => {
                unsafe {
                    let event = xlib::XKeyEvent::from(ev);
                    if self.on_keypress( &event ) {
                        return true;
                    }
                }
            },

            _ => {},
        }
        false
    }

    pub fn flush(&self) {
        unsafe {
            xlib::XFlush(self.display);
        }
    }

    unsafe fn on_client_message( &mut self, event: &xlib::XClientMessageEvent ) {
//        let data: [i32; 5] = [
//            event.data.get_long(0) as i32,
//            event.data.get_long(1) as i32,
//            event.data.get_long(2) as i32,
//            event.data.get_long(3) as i32,
//            event.data.get_long(4) as i32, ];
    }

    unsafe fn on_map_request( &mut self, event: &mut xlib::XMapRequestEvent ) {
        let screen = xlib::XDefaultScreenOfDisplay( self.display );
        let visual = xlib::XDefaultVisual( self.display, xlib::XDefaultScreen( self.display) );
        let mut wa = xlib::XWindowAttributes {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            border_width: 0,
            depth: 0,
            visual: visual,
            root: 0,
            class: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: 0,
            colormap: 0,
            map_installed: 0,
            map_state: 0,
            all_event_masks: 0,
            your_event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: 0,
            screen: screen,
        };

        if xlib::XGetWindowAttributes( self.display, event.window, &mut wa ) == 0 {
            return;
        }

        if wa.override_redirect == 1 {
            return;
        }

        let mut wc = xlib::XWindowChanges {
            x: wa.x,
            y: wa.y,
            width: wa.width,
            height: wa.height,
            border_width: 10,
            sibling: 0,
            stack_mode: 0,
        };

        xlib::XConfigureWindow( self.display, event.window, xlib::CWBorderWidth as u32, &mut wc );
        xlib::XSetWindowBorder( self.display, event.window, wc.border_width as u64 );
        xlib::XMoveResizeWindow( self.display, event.window, wc.x, wc.y, wc.width as u32, wc.height as u32 );
        xlib::XSelectInput( self.display, event.window,
                    xlib::EnterWindowMask|
                    xlib::FocusChangeMask|
                    xlib::PropertyChangeMask|
                    xlib::StructureNotifyMask );
        xlib::XMapWindow( self.display, event.window );
    }

    unsafe fn on_keypress( &mut self, event: &xlib::XKeyEvent ) -> bool {
        use keys::*;
        let key = xlib::XKeysymToString(
            xlib::XKeycodeToKeysym( self.display, event.keycode as u8, 0 ) );
        let key = CString::from_raw(key);

        let key_info = Key::new( key.to_str().unwrap(), event.state );

        // Handle key events
        match key_info {
            EXIT_KEY => {
                println!( "Exiting!" );
                return true;
            },

            _ => {},
        }

        false
    }

    unsafe fn on_button_press( &mut self, event: &xlib::XButtonEvent ) {
        let button_info = keys::MouseCmd::new( event.button, event.state );

        match button_info {
            keys::MOUSE_MOVE => {
                if event.subwindow != 0 {
                    println!("Pointer grabbed");
                    xlib::XGrabPointer( self.display, event.subwindow, 1,
                        (xlib::PointerMotionMask|xlib::ButtonPressMask) as u32,
                        xlib::GrabModeAsync, xlib::GrabModeAsync,
                        0, 0, event.time);
                }
            },

            keys::MOUSE_RAISE => {
                if event.subwindow != 0 {
                    xlib::XRaiseWindow( self.display, event.subwindow );
                }
            },

            _ => {},
        }
    }

    unsafe fn on_button_release( &mut self, event: &xlib::XButtonEvent ) {
        let button_info = keys::MouseCmd::new( event.button, event.state );

        println!("Button Info: {} {} {}", button_info.button, button_info.modifier, event.time );

        match button_info {
            keys::MOUSE_MOVE => {
                xlib::XUngrabPointer( self.display, event.time );
                println!("Pointer released");
            },

            _ => {},
        }
    }
}
