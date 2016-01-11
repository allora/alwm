use std::thread::spawn;
use std::process::Command;
use std::num::Wrapping;
use std::cmp::max;
use libc::{c_uchar,c_int, c_ulong};
use std::ffi::CString;
use std::ptr::{
  null,
  null_mut,
};

use x11::xlib;
use config;
use config::KeyCmd;

unsafe extern fn error_handler(_: *mut xlib::Display, _: *mut xlib::XErrorEvent) -> c_int {
    return 0;
}

pub struct WindowSystem {
    display:    *mut xlib::Display,
    root:       xlib::Window,
    x:          i32,
    y:          i32,
    w:          u32,
    h:          u32,
    button_id:  u32,
    borderinfo: config::BorderInfo,
    focuswin:   xlib::Window,
}

impl WindowSystem {
    pub fn new() -> WindowSystem {
        use x11::xlib::*;

        let borderinfo = config::BorderInfo::new( config::FOCUS_BORDERS, config::UNFOCUSED_BORDERS );

        unsafe {
            // Open display
            let display = xlib::XOpenDisplay(null());
            if display == null_mut() {
                panic!("Exiting: Cannot find display");
            }

            // Create window
            let screen = xlib::XDefaultScreenOfDisplay(display);
            let root = xlib::XRootWindowOfScreen(screen);

            xlib::XSetErrorHandler(Some(error_handler));

            let ws = WindowSystem {
                display: display,
                root: root,
                x: 0,
                y: 0,
                w: 0,
                h: 0,
                button_id: 0,
                borderinfo: borderinfo,
                focuswin: root,
            };

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

            // Set up window events
            XChangeWindowAttributes( ws.display, ws.root, CWEventMask|CWCursor, &mut wa );
            XSelectInput( ws.display, ws.root, wa.event_mask );
            xlib::XSync( ws.display, 0 );
            xlib::XUngrabButton(ws.display, 0, 0x8000, ws.root);

            let name = (*CString::new(&b"ALWM"[..]).unwrap()).as_ptr();

            let wmcheck = ws.get_atom("_NET_SUPPORTING_WM_CHECK");
            let wmname = ws.get_atom("_NET_WM_NAME");
            let utf8 = ws.get_atom("UTF8_STRING");
            let xa_window = ws.get_atom("xlib::XA_WINDOW");

            let mut root_cpy = ws.root;
            let root_ptr : *mut Window = &mut root_cpy;
            xlib::XChangeProperty(ws.display, ws.root, wmcheck, xa_window, 32, 0, root_ptr as *mut c_uchar, 1);
            xlib::XChangeProperty(ws.display, ws.root, wmname, utf8, 8, 0, name as *mut c_uchar, 5);

            ws
        }
    }

    pub fn grab_keys(&self) {
        unsafe {
            // Grab keys
            // Exit key behavior
            let kc_exit = xlib::XKeysymToKeycode( self.display, KeyCmd::get_keysym( config::EXIT_KEY ) );
            xlib::XGrabKey( self.display, kc_exit as i32, KeyCmd::get_modifier( config::EXIT_KEY ),
                self.root as c_ulong, 1, xlib::GrabModeAsync, xlib::GrabModeAsync );
            let kc_term = xlib::XKeysymToKeycode( self.display, KeyCmd::get_keysym( config::TERM_KEY ) );
            xlib::XGrabKey( self.display, kc_term as i32, KeyCmd::get_modifier( config::TERM_KEY ),
                self.root as c_ulong, 1, xlib::GrabModeAsync, xlib::GrabModeAsync );
            let kc_run = xlib::XKeysymToKeycode( self.display, KeyCmd::get_keysym( config::RUN_KEY ) );
            xlib::XGrabKey( self.display, kc_run as i32, KeyCmd::get_modifier( config::RUN_KEY ),
                self.root as c_ulong, 1, xlib::GrabModeAsync, xlib::GrabModeAsync );
        }
    }

    pub fn grab_buttons(&self) {
        unsafe {
            // Grab mouse
            xlib::XGrabButton( self.display, config::MOUSE_MOVE.button, config::MOUSE_MOVE.modifier,
                self.root, 1, xlib::ButtonPressMask as u32, xlib::GrabModeAsync, xlib::GrabModeAsync, 0, 0 );
            xlib::XGrabButton( self.display, config::MOUSE_RESIZE.button, config::MOUSE_RESIZE.modifier,
                self.root, 1, xlib::ButtonPressMask as u32, xlib::GrabModeAsync, xlib::GrabModeAsync, 0, 0 );
            xlib::XGrabButton( self.display, config::MOUSE_RAISE.button, config::MOUSE_RAISE.modifier,
                self.root, 1, xlib::ButtonPressMask as u32, xlib::GrabModeAsync, xlib::GrabModeAsync, 0, 0 );
        }
    }

    fn get_atom(&self, s: &str) -> u64 {
        unsafe {
            match CString::new(s) {
                Ok(b) => xlib::XInternAtom(self.display, b.as_ptr(), 0) as u64,
                _     => panic!("Invalid atom! {}", s)
            }
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
                self.on_button_press( &event );
            },

            xlib::ButtonRelease => {
                let event = xlib::XButtonEvent::from(ev);
                self.on_button_release( &event );
            },

//            xlib::ClientMessage => {
//                let event = xlib::XClientMessageEvent::from(ev);
//                self.on_client_message( &event );
//            },

            xlib::ConfigureNotify => {
                let event = xlib::XConfigureEvent::from(ev);

                unsafe {
                    xlib::XClearWindow( self.display, self.root );
                }

                if event.window != self.root {
                    if event.window == self.focuswin {
                        self.draw_borders( true, event.window );
                    }
                    else {
                        self.draw_borders( false, event.window );
                    }
                }
            }

            xlib::DestroyNotify => {
            },

            xlib::EnterNotify => {
                let event = xlib::XEnterWindowEvent::from(ev);
                self.on_enter_notify( &event );
            }

            xlib::MapRequest => {
                let mut event = xlib::XMapRequestEvent::from(ev);
                self.on_map_request( &mut event );
            },

//            xlib::CreateNotify => {
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

    fn focus( &mut self, window: xlib::Window, time: c_ulong ) {
        if self.focuswin != window {
            unsafe{
                xlib::XSetInputFocus( self.display, window, xlib::RevertToParent, time );
                if !config::SLOPPYFOCUS {
                    xlib::XRaiseWindow( self.display, window );
                }
            }
        }
        self.draw_borders( true, window );
        self.focuswin = window;
    }

    fn draw_borders( &mut self, isfocused: bool, window: xlib::Window ) {
        if self.root as u64 == window as u64 { return; }
        let mut borders = config::UNFOCUSED_BORDERS;
        let mut count = config::NUM_UNFOCUSED_BORDERS;
        let mut size = self.borderinfo.get_unfocus_size();
        if isfocused {
            borders = config::FOCUS_BORDERS;
            count = config::NUM_FOCUSED_BORDERS;
            size = self.borderinfo.get_focus_size();
        }

        unsafe {
//            let mut wc = xlib::XWindowChanges {
//                x: 0,
//                y: 0,
//                width: 0,
//                height: 0,
//                border_width: size * 2,
//                sibling: 0,
//                stack_mode: 0,
//            };
//
//            xlib::XConfigureWindow( self.display, window, xlib::CWBorderWidth as u32, &mut wc );

            let mut gcv = xlib::XGCValues {
                function: 0,
                plane_mask: 0,
                foreground: 0,
                background: 0,
                line_width: 0,
                line_style: 0,
                cap_style: 0,
                join_style: 0,
                fill_style: 0,
                fill_rule: 0,
                arc_mode: 0,
                tile: 0,
                stipple: 0,
                ts_x_origin: 0,
                ts_y_origin: 0,
                font: 0,
                subwindow_mode: 0,
                graphics_exposures: 0,
                clip_x_origin: 0,
                clip_y_origin: 0,
                clip_mask: 0,
                dash_offset: 0,
                dashes: 0,
            };
            let mut wa = self.get_empty_wa();
            xlib::XGetWindowAttributes( self.display, window, &mut wa );

            let cmap = xlib::XDefaultColormap( self.display, 0 );
            let pixmap = xlib::XCreatePixmap( self.display, self.root,
                                              (wa.x - size) as u32, (wa.y - size) as u32, wa.depth as u32 );

            for i in 0 .. count {

                let new_x = wa.x - size;
                let new_y = wa.y - size;
                let new_w = wa.width + ( 2 * size );
                let new_h = wa.height + ( 2 * size );


                let mut color = xlib::XColor {
                    pixel: 0,
                    red: 0,
                    green: 0,
                    blue: 0,
                    flags: 0,
                    pad: 0,
                };

                let gc = xlib::XCreateGC( self.display, pixmap, 0, &mut gcv );
                let color_string = CString::new( borders[i].color).unwrap().into_raw();

                xlib::XParseColor( self.display, cmap, color_string, &mut color);
                xlib::XAllocColor( self.display, cmap, &mut color );

                let _ = CString::from_raw(color_string);

                xlib::XSetForeground( self.display, gc, color.pixel );
                xlib::XFillRectangle( self.display, pixmap, gc, new_x, new_y, new_w as u32, new_h as u32 );
                xlib::XSync( self.display, 0 );

                xlib::XFreeGC( self.display, gc );
                size = size - borders[i].size;
            }

            let mut s_wa = xlib::XSetWindowAttributes {
                background_pixmap: 0,
                background_pixel: 0,
                border_pixmap: pixmap,
                border_pixel: 0,
                bit_gravity: 0,
                win_gravity: 0,
                backing_store: 0,
                backing_planes: 0,
                backing_pixel: 0,
                save_under: 0,
                event_mask: 0,
                do_not_propagate_mask: 0,
                override_redirect: 0,
                colormap: 0,
                cursor: 0,
            };

            xlib::XChangeWindowAttributes( self.display, window,
                        xlib::CWBorderPixmap, &mut s_wa );
            xlib::XSync( self.display, 0 );

            xlib::XFreePixmap( self.display, pixmap );
            xlib::XFreeColormap( self.display, cmap );
            self.flush();
        }
    }

    fn on_enter_notify( &mut self, event: &xlib::XEnterWindowEvent ) {
        if config::SLOPPYFOCUS {
            self.focus( event.window, event.time );
        }
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
                border_width: 0,
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

            self.draw_borders( false, event.window );
            xlib::XMapWindow( self.display, event.window );
        }

    }

    fn on_keypress( &mut self, event: &xlib::XKeyEvent ) -> bool {
        use config::*;

        let mut open_term = false;
        let mut open_run = false;

        unsafe {
            let key = xlib::XKeysymToString(
                xlib::XKeycodeToKeysym( self.display, event.keycode as u8, 0 ) );
            let key = CString::from_raw(key);

            let key_info = KeyCmd::new( key.to_str().unwrap(), event.state );

            // Handle key events
            match key_info {
                EXIT_KEY => {
                    return true;
                },
                TERM_KEY => {
                    open_term = true;
                },
                RUN_KEY => {
                    open_run = true;
                },

                _ => {},
            }
        }

        if open_run {
            spawn(move || {
                Command::new( RUN ).spawn().unwrap_or_else( |e| {
                    panic!("Invalid run command {}", e)});
            });
        }

        if open_term {
            spawn(move || {
                Command::new( TERMINAL ).spawn().unwrap_or_else( |e| {
                    panic!("Inavlid terminal command {}", e)});
            });
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
                    self.focus( event.subwindow, event.time );
                    self.on_resize_move( &event );
                }
            },

            config::MOUSE_MOVE => {
                if event.subwindow != 0 {
                    self.focus( event.subwindow, event.time );
                    self.on_resize_move( &event );
                }
            },

            config::MOUSE_RAISE => {
                if event.subwindow != 0 {
                    self.focus( event.subwindow, event.time );
                }
            },

            _ => {},
        }
    }

    fn on_button_release( &mut self, event: &xlib::XButtonEvent ) {
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
