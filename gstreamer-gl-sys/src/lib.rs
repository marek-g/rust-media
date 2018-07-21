#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

extern crate libc;
extern crate glib_sys as glib;
extern crate gstreamer_sys as gst;
extern crate gstreamer_base_sys as gst_base;

#[allow(unused_imports)]
use libc::{c_int, c_char, c_uchar, c_float, c_uint, c_double,
    c_short, c_ushort, c_long, c_ulong,
    c_void, size_t, ssize_t, intptr_t, uintptr_t, time_t, FILE};

#[allow(unused_imports)]
use glib::{gboolean, gconstpointer, gpointer, GType, Volatile};

use gst::GstContext;

#[repr(C)]
pub struct GstGLDisplay(c_void);

#[repr(C)]
pub struct GstGLDisplayX11(c_void);

extern "C" {
    pub fn gst_gl_display_new() -> *mut GstGLDisplay;

    pub fn gst_gl_display_x11_new(name: *const c_char) -> *mut GstGLDisplayX11;
    //pub fn gst_gl_display_x11_new_with_display(display: *mut x11::Display) -> *mut GstGLDisplayX11;

    pub fn gst_context_new(context_type: *const c_char, persistent: gboolean) -> *mut GstContext;

    pub fn gst_context_set_gl_display(context: *mut GstContext, display: *mut GstGLDisplay);
}
