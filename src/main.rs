extern crate libc;
extern crate x11;

pub mod config;
pub mod windowsystem;

use windowsystem::WindowSystem;

#[allow(while_true)]
fn main() {
    let mut window_system = WindowSystem::new();

    window_system.on_init();

    let mut exit_event = false;
    while !exit_event {
        exit_event = window_system.on_update();
    }
}
