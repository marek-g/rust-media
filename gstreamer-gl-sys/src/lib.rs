#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

extern crate libc;
extern crate glib_sys as glib;
extern crate gstreamer_sys as gst;
extern crate gstreamer_base_sys as gst_base;
extern crate gtypes;

#[cfg(target_os="linux")]
extern crate x11_dl;

#[allow(unused_imports)]
use libc::{c_int, c_char, c_uchar, c_float, c_uint, c_double,
    c_short, c_ushort, c_long, c_ulong,
    c_void, size_t, ssize_t, intptr_t, uintptr_t, time_t, FILE};

#[allow(unused_imports)]
use glib::{gboolean, gconstpointer, gpointer, GType};

use gst::GstContext;
use gst::GstStructure;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GstGLDisplay(*mut c_void);

unsafe impl Sync for GstGLDisplay {}
unsafe impl Send for GstGLDisplay {}

impl GstGLDisplay {
    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.0
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GstGLDisplayX11(*mut c_void);

unsafe impl Sync for GstGLDisplayX11 {}
unsafe impl Send for GstGLDisplayX11 {}

impl GstGLDisplayX11 {
    pub fn as_display(&self) -> GstGLDisplay {
        GstGLDisplay(self.0)
    }

    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.0
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GstGLContext(*mut c_void);

unsafe impl Sync for GstGLContext {}
unsafe impl Send for GstGLContext {}

impl GstGLContext {
    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.0
    }
}

pub type GstGLPlatform = c_int;
pub const GST_GL_PLATFORM_NONE: GstGLPlatform = 0;
pub const GST_GL_PLATFORM_EGL: GstGLPlatform = 1;
pub const GST_GL_PLATFORM_GLX: GstGLPlatform = 2;
pub const GST_GL_PLATFORM_WGL: GstGLPlatform = 4;
pub const GST_GL_PLATFORM_CGL: GstGLPlatform = 8;
pub const GST_GL_PLATFORM_EAGL: GstGLPlatform = 16;
pub const GST_GL_PLATFORM_ANY: GstGLPlatform = 4294967295;

pub type GstGLAPI = c_int;
pub const GST_GL_API_NONE: GstGLAPI = 0;
pub const GST_GL_API_OPENGL: GstGLAPI = 1;
pub const GST_GL_API_OPENGL3: GstGLAPI = 2;
pub const GST_GL_API_GLES1: GstGLAPI = 32768;
pub const GST_GL_API_GLES2: GstGLAPI = 65536;
pub const GST_GL_API_ANY: GstGLPlatform = 4294967295;

extern "C" {
    pub fn gst_gl_display_new() -> GstGLDisplay;

    #[cfg(target_os="linux")]
    pub fn gst_gl_display_x11_new(name: *const c_char) -> GstGLDisplayX11;
    #[cfg(target_os="linux")]
    pub fn gst_gl_display_x11_new_with_display(display: *mut x11_dl::xlib::Display) -> GstGLDisplayX11;

    pub fn gst_gl_context_new(display: *mut c_void) -> GstGLContext;
    //pub fn gst_gl_context_new_wrapped(display: GstGLDisplay, handle: gtypes::primitive::guintptr,
    //    context_type: GstGLPlatform, available_apis: GstGLAPI) -> GstGLContext;
    pub fn gst_gl_context_new_wrapped(display: *mut c_void, handle: gtypes::primitive::guintptr,
        context_type: GstGLPlatform, available_apis: GstGLAPI) -> GstGLContext;
    pub fn gst_gl_context_get_type() -> GType;
    pub fn gst_gl_context_get_gl_platform(context: GstGLContext) -> GstGLPlatform;
    pub fn gst_gl_context_get_gl_api(context: GstGLContext) -> GstGLAPI;

    pub fn gst_context_new(context_type: *const c_char, persistent: gboolean) -> *mut GstContext;
    pub fn gst_context_set_gl_display(context: *mut GstContext, display: *mut c_void); //display: GstGLDisplay);
    pub fn gst_structure_set(structure: *mut GstStructure, field_name: *const c_char, ...);
    pub fn gst_element_set_context(element: *mut c_void, context: *mut GstContext);
}
