use std::num::Wrapping;
use std::cmp::max;
use libc::c_ulong;
use std::ffi::CString;
use std::ptr::{
  null,
  null_mut,
};

use x11::xlib;
use config;
use config::KeyCmd;

pub struct WindowSystem {
    display:    *mut xlib::Display,
    root:       xlib::Window,
    x:          i32,
    y:          i32,
    w:          u32,
    h:          u32,
    button_id:  u32,
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
                x: 0,
                y: 0,
                w: 0,
                h: 0,
                button_id: 0,
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
                        StructureNotifyMask|
                        ButtonPressMask|
                        ButtonReleaseMask|
                        PropertyChangeMask,
        };

        unsafe {

            // Grab keys
            // Exit key behavior
            let kc_exit = xlib::XKeysymToKeycode( self.display, KeyCmd::get_keysym( config::EXIT_KEY ) );
            xlib::XGrabKey( self.display, kc_exit as i32, KeyCmd::get_modifier( config::EXIT_KEY ),
                self.root as c_ulong, 1, xlib::GrabModeAsync, xlib::GrabModeAsync );

            // Grab mouse
            xlib::XGrabButton( self.display, config::MOUSE_MOVE.button, config::MOUSE_MOVE.modifier,
                self.root, 1, xlib::ButtonPressMask as u32, xlib::GrabModeAsync, xlib::GrabModeAsync, 0, 0 );
            xlib::XGrabButton( self.display, config::MOUSE_RESIZE.button, config::MOUSE_RESIZE.modifier,
                self.root, 1, xlib::ButtonPressMask as u32, xlib::GrabModeAsync, xlib::GrabModeAsync, 0, 0 );
            xlib::XGrabButton( self.display, config::MOUSE_RAISE.button, config::MOUSE_RAISE.modifier,
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
                unsafe {
                    while xlib::XCheckTypedEvent( self.display, xlib::MotionNotify, &mut ev ) == 1 {};
                }

                let event = xlib::XMotionEvent::from(ev);
                self.on_motion( &event );
            },

            xlib::ButtonPress => {
                let event = xlib::XButtonEvent::from(ev);
                println!("Button Pressed: {}", event.button);
                self.on_button_press( &event );
            },

            xlib::ButtonRelease => {
                let event = xlib::XButtonEvent::from(ev);
                self.on_button_release( &event );
            },

            xlib::ClientMessage => {
                let event = xlib::XClientMessageEvent::from(ev);
                self.on_client_message( &event );
            },

            xlib::DestroyNotify => {
                println!("Got destroy notify");
            },

            xlib::MapRequest => {
                let mut event = xlib::XMapRequestEvent::from(ev);
                self.on_map_request( &mut event );
            },

//            xlib::CreateNotify => {
//                println!("Got create notify");
//            }

            xlib::KeyPress => {
                let event = xlib::XKeyEvent::from(ev);
                if self.on_keypress( &event ) {
                    return true;
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

    fn on_client_message( &mut self, event: &xlib::XClientMessageEvent ) {
        println!("Client message: {}", event.window);
    }

    fn on_map_request( &mut self, event: &mut xlib::XMapRequestEvent ) {
        unsafe {
            let mut wa = self.get_empty_wa();

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
    }

    fn on_keypress( &mut self, event: &xlib::XKeyEvent ) -> bool {
        use config::*;

        unsafe {
            let key = xlib::XKeysymToString(
                xlib::XKeycodeToKeysym( self.display, event.keycode as u8, 0 ) );
            let key = CString::from_raw(key);

            let key_info = KeyCmd::new( key.to_str().unwrap(), event.state );

            // Handle key events
            match key_info {
                EXIT_KEY => {
                    println!( "Exiting!" );
                    return true;
                },

                _ => {},
            }
        }
        false
    }

    fn on_resize_move( &mut self, event: &xlib::XButtonEvent ) {
        if event.button == 1 {
            self.x = event.x_root;
            self.y = event.y_root;
        }
        if event.button == 3 {
            self.w = event.x_root as u32;
            self.h = event.y_root as u32;
        }

        self.button_id = event.button;
        println!("Pointer grabbed");
        unsafe {
            xlib::XGrabPointer( self.display, event.subwindow, 1,
                (xlib::PointerMotionMask|xlib::ButtonReleaseMask) as u32,
                xlib::GrabModeAsync, xlib::GrabModeAsync,
                0, 0, event.time);
        }
    }

    fn on_button_press( &mut self, event: &xlib::XButtonEvent ) {
        let button_info = config::MouseCmd::new( event.button, event.state );

        match button_info {
            config::MOUSE_RESIZE => {
                if event.subwindow != 0 {
                    println!("Resize");
                    unsafe {
                        xlib::XRaiseWindow( self.display, event.subwindow );
                    }
                    self.on_resize_move( &event );
                }
            },

            config::MOUSE_MOVE => {
                if event.subwindow != 0 {
                    println!("Move");
                    unsafe {
                        xlib::XRaiseWindow( self.display, event.subwindow );
                    }
                    self.on_resize_move( &event );
                }
            },

            config::MOUSE_RAISE => {
                if event.subwindow != 0 {
                    unsafe {
                        xlib::XRaiseWindow( self.display, event.subwindow );
                    }
                }
            },

            _ => {},
        }

        println!("End button press function");
    }

    fn on_button_release( &mut self, event: &xlib::XButtonEvent ) {
        println!("Ungrab pointer");
        unsafe {
            xlib::XUngrabPointer( self.display, event.time );
        }
    }

    fn on_motion( &mut self, event: &xlib::XMotionEvent ) {
        if self.button_id == 1 {
            self.on_move( &event );
        }
        if self.button_id == 3 {
            self.on_resize( &event );
        }

        self.flush();
    }

    fn on_resize( &mut self, event: &xlib::XMotionEvent ) {
        unsafe {
            let mut wa = self.get_empty_wa();
            if xlib::XGetWindowAttributes( self.display, event.window, &mut wa ) == 0 {
                return;
            }

            let xdiff = Wrapping::<u32>( event.x_root as u32 ) - Wrapping::<u32>( self.w );
            let ydiff = Wrapping::<u32>( event.y_root as u32 ) - Wrapping::<u32>( self.h );

            self.w = event.x_root as u32;
            self.h = event.y_root as u32;

            let new_w = Wrapping::<u32>(wa.width as u32) + xdiff;
            let new_h = Wrapping::<u32>(wa.height as u32) + ydiff;

            let new_w = max(1, new_w.0);
            let new_h = max(1, new_h.0);

            println!("Coord Diffs: {}, {}", xdiff.0, ydiff.0);
            println!("Coords: {}, {}", new_w, new_h );
            xlib::XResizeWindow( self.display, event.window, new_w, new_h );
        }
    }

    fn on_move( &mut self, event: &xlib::XMotionEvent ) {
        unsafe {
            let mut wa = self.get_empty_wa();
            if xlib::XGetWindowAttributes( self.display, event.window, &mut wa ) == 0 {
                return;
            }

            let xdiff = event.x_root - self.x;
            let ydiff = event.y_root - self.y;

            self.x = event.x_root;
            self.y = event.y_root;

            let new_x = wa.x + xdiff;
            let new_y = wa.y + ydiff;

            println!("Coords: {}, {}", new_x, new_y );
            xlib::XMoveWindow( self.display, event.window, new_x, new_y );
        }
    }

    unsafe fn get_empty_wa ( &self ) -> xlib::XWindowAttributes {
        let screen = xlib::XDefaultScreenOfDisplay( self.display );
        let visual = xlib::XDefaultVisual( self.display, xlib::XDefaultScreen( self.display ) );
        xlib::XWindowAttributes {
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
        }
    }
}
