# Allie's Lazy WM

ALWM is my attempt at learning Rust through a toy window manager. This project is many firsts for me. My first xlib project, my first WM, and one of my first Rust projects. As I find time and learn more, new features will be added.

ALWM is inspired by dwm and wtfwm code bases.

Current Features:
* Exit funcitonality through the default key binding Super+Shift+F1
* Window borders! (Hardcoded atm)
* Window Focus
* Window Movement
* Window Resize
* Launch external term defined by user
* Launch external run command (default to dmenu_run)

Work In Progress:
* Add debugging helpers
* Window decoration configurable through config.rs (similar to config.h in dwm)
* Code clean up (SO MANY UNSAFE D:)
