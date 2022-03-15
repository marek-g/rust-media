extern crate glutin;
extern crate gstreamer as gst;
extern crate gstreamer_app as gst_app;
extern crate gstreamer_gl_sys as gst_gl_sys;
extern crate winit;

use self::glutin::platform::ContextTraitExt;
use self::glutin::PossiblyCurrent;
use self::gst::prelude::*;
use self::gst::Context;
use self::gst::MessageView;
use glib::translate::ToGlibPtr;
use std::sync::Arc;

#[cfg(target_os = "linux")]
use self::glutin::platform::unix::RawHandle::Glx;
#[cfg(target_os = "linux")]
use self::winit::platform::unix::x11::XConnection;

#[cfg(target_os = "windows")]
use self::glutin::platform::windows::RawHandle::Wgl;


/// Wrapper for GstGLDisplay that can be Send.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GstGLDisplaySend(*mut gst_gl_sys::GstGLDisplay);

unsafe impl Sync for GstGLContextSend {}
unsafe impl Send for GstGLContextSend {}

impl GstGLDisplaySend {
    pub fn as_mut_ptr(&self) -> *mut gst_gl_sys::GstGLDisplay {
            self.0
    }
}


/// Wrapper for GstGLContext that can be Send.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GstGLContextSend(*mut gst_gl_sys::GstGLContext);

unsafe impl Sync for GstGLDisplaySend {}
unsafe impl Send for GstGLDisplaySend {}

impl GstGLContextSend {
    pub fn as_mut_ptr(&self) -> *mut gst_gl_sys::GstGLContext {
        self.0
    }
}




pub fn create_pipeline_videotest() -> (gst::Pipeline, gst_app::AppSink) {
    let source = gst::ElementFactory::make("videotestsrc", Some("source"))
        .expect("Could not create source element.");
    source.set_property_from_str("pattern", "smpte");

    let video_sink =
        gst::ElementFactory::make("appsink", Some("sink")).expect("Could not create sink element");
    let video_app_sink = video_sink.dynamic_cast::<gst_app::AppSink>().unwrap();
    video_app_sink.set_caps(Some(&gst::Caps::new_simple(
        "video/x-raw",
        &[
            ("format", &"BGRA"),
            ("pixel-aspect-ratio", &gst::Fraction::from((1, 1))),
        ],
    )));

    let video_sink = video_app_sink.dynamic_cast::<gst::Element>().unwrap();

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // Build the pipeline
    pipeline.add_many(&[&source, &video_sink]).unwrap();
    source
        .link(&video_sink)
        .expect("Elements could not be linked.");

    let video_app_sink = video_sink.dynamic_cast::<gst_app::AppSink>().unwrap();

    (pipeline, video_app_sink)
}

pub fn create_appsink_pipeline_url(url: &str) -> (gst::Pipeline, gst_app::AppSink) {
    let source = gst::ElementFactory::make("uridecodebin", Some("source"))
        .expect("Could not create uridecodebin element.");
    source.set_property_from_str("uri", url);

    let video_convert = gst::ElementFactory::make("videoconvert", Some("videoconvert"))
        .expect("Could not create videoconvert element.");
    let audio_convert = gst::ElementFactory::make("audioconvert", Some("audioconvert"))
        .expect("Could not create audioconvert element.");

    let video_sink = gst::ElementFactory::make("appsink", Some("videosink"))
        .expect("Could not create sink element");
    let audio_sink = gst::ElementFactory::make("autoaudiosink", Some("audiosink"))
        .expect("Could not create sink element.");

    let video_app_sink = video_sink.dynamic_cast::<gst_app::AppSink>().unwrap();
    video_app_sink.set_caps(Some(&gst::Caps::new_simple(
        "video/x-raw",
        &[
            ("format", &"BGRA"),
            ("pixel-aspect-ratio", &gst::Fraction::from((1, 1))),
        ],
    )));

    let video_sink = video_app_sink.dynamic_cast::<gst::Element>().unwrap();

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // Build the pipeline
    pipeline
        .add_many(&[
            &source,
            &video_convert,
            &video_sink,
            &audio_convert,
            &audio_sink,
        ])
        .unwrap();
    video_convert
        .link(&video_sink)
        .expect("Elements could not be linked.");
    audio_convert
        .link(&audio_sink)
        .expect("Elements could not be linked.");

    // Connect the pad-added signal
    let pipeline_clone = pipeline.clone();
    //let convert_clone = convert.clone();
    let video_sink_clone = video_convert.clone();
    let audio_sink_clone = audio_convert.clone();
    source.connect_pad_added(move |_, src_pad| {
        let pipeline = &pipeline_clone;
        let video_sink = &video_sink_clone;
        let audio_sink = &audio_sink_clone;

        println!(
            "Received new pad {} from {}",
            src_pad.name(),
            pipeline.name()
        );

        let new_pad_caps = src_pad
            .current_caps()
            .expect("Failed to get caps of new pad.");
        let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get first structure of caps.");
        let new_pad_type = new_pad_struct.name();

        println!("src pad type: {}", new_pad_type);

        let is_audio = new_pad_type.starts_with("audio/x-raw");
        let is_video = new_pad_type.starts_with("video/x-raw");

        if is_video {
            let sink_pad = video_sink
                .static_pad("sink")
                .expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                println!("We are already linked. Ignoring.");
                return;
            }

            let ret = src_pad.link(&sink_pad);
            if let Ok(_) = ret {
                println!("Type is {} but link failed.", new_pad_type);
            } else {
                println!("Link succeeded (type {}).", new_pad_type);
            }
        }

        if is_audio {
            let sink_pad = audio_sink
                .static_pad("sink")
                .expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                println!("We are already linked. Ignoring.");
                return;
            }

            let ret = src_pad.link(&sink_pad);
            if let Ok(_) = ret {
                println!("Type is {} but link failed.", new_pad_type);
            } else {
                println!("Link succeeded (type {}).", new_pad_type);
            }
        }
    });

    let video_app_sink = video_sink.dynamic_cast::<gst_app::AppSink>().unwrap();

    (pipeline, video_app_sink)
}

#[cfg(target_os = "linux")]
pub fn create_opengl_pipeline_url(
    url: &str,
    context: &glutin::Context<PossiblyCurrent>,
    xconnection: &Arc<XConnection>,
) -> (gst::Pipeline, gst::Element) {
    let source = gst::ElementFactory::make("uridecodebin", Some("source"))
        .expect("Could not create uridecodebin element.");
    source.set_property_from_str("uri", url);

    //let video_convert = gst::ElementFactory::make("videoconvert", "videoconvert")
    //  .expect("Could not create videoconvert element.");
    let audio_convert = gst::ElementFactory::make("audioconvert", Some("audioconvert"))
        .expect("Could not create audioconvert element.");

    let video_sink = gst::ElementFactory::make("glimagesink", Some("videosink"))
        .expect("Could not create sink element");
    let audio_sink = gst::ElementFactory::make("autoaudiosink", Some("audiosink"))
        .expect("Could not create sink element.");

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // get display & context handles
    let gl_context = unsafe {
        let context = context.raw_handle();
        if let Glx(glx_context) = context {
            glx_context as usize
        } else {
            unimplemented!()
        }
    };
    let display = xconnection.display;

    let gst_display = unsafe {
        GstGLDisplaySend(gstreamer_gl_x11_sys::gst_gl_display_x11_new_with_display(display as glib::ffi::gpointer) as *mut gst_gl_sys::GstGLDisplay)
    };

    // gst_sdl_context =
    //  gst_gl_context_new_wrapped (sdl_gl_display, (guintptr) sdl_gl_context,
    //  gst_gl_platform_from_string (platform), GST_GL_API_OPENGL);
    let gst_context = unsafe {
        GstGLContextSend(gst_gl_sys::gst_gl_context_new_wrapped(
            gst_display.as_mut_ptr(),
            gl_context as usize,
            gst_gl_sys::GST_GL_PLATFORM_GLX,
            gst_gl_sys::GST_GL_API_OPENGL,
        ))
    };

    unsafe {
        println!(
            "     gl_display: {}, first_bytes: {} {} {} {}",
            display as usize,
            *((display as usize + 0) as *const usize),
            *((display as usize + 8) as *const usize),
            *((display as usize + 16) as *const usize),
            *((display as usize + 24) as *const usize),
        );
        println!(
            "    gst_display: {}, first_bytes: {} {} {} {}",
            gst_display.as_mut_ptr() as usize,
            *((gst_display.as_mut_ptr() as usize + 0) as *const usize),
            *((gst_display.as_mut_ptr() as usize + 8) as *const usize),
            *((gst_display.as_mut_ptr() as usize + 16) as *const usize),
            *((gst_display.as_mut_ptr() as usize + 24) as *const usize),
        );

        println!(
            "     gl_context: {}, first_bytes: {} {} {} {}",
            gl_context as usize,
            *((gl_context as usize + 0) as *const usize),
            *((gl_context as usize + 8) as *const usize),
            *((gl_context as usize + 16) as *const usize),
            *((gl_context as usize + 24) as *const usize),
        );
        println!(
            "     gst_context: {}, first_bytes: {} {} {} {}",
            gst_context.as_mut_ptr() as usize,
            *((gst_context.as_mut_ptr() as usize + 0) as *const usize),
            *((gst_context.as_mut_ptr() as usize + 8) as *const usize),
            *((gst_context.as_mut_ptr() as usize + 16) as *const usize),
            *((gst_context.as_mut_ptr() as usize + 24) as *const usize),
        );
    }

    let bus = pipeline.bus().unwrap();
    bus.add_signal_watch();
    bus.enable_sync_message_emission();
    bus.connect_sync_message(None, move |_bus, message| {
        match message.view() {
            MessageView::NeedContext(need_context) => {
                let context_type = need_context.context_type();
                let src = message
                    .src()
                    .unwrap()
                    .dynamic_cast::<gst::Element>()
                    .unwrap();
                println!("need context: {:?}, src: {:?}", context_type, src);

                // TODO: add binding for gst_gl_context_new_wrapped from /usr/lib/libgstgl-1.0
                // (I've got GstGL-1.0.gir for it!)
                //
                // sdl_gl_display = gst_gl_display_new ();
                // sdl_gl_display = (GstGLDisplay *) gst_gl_display_x11_new_with_display (sdl_display);
                //
                // gst_sdl_context =
                //      gst_gl_context_new_wrapped (sdl_gl_display, (guintptr) sdl_gl_context,
                //      gst_gl_platform_from_string (platform), GST_GL_API_OPENGL);

                if context_type == "gst.gl.GLDisplay" {
                    /*unsafe {
                        let context = gst_gl_sys::gst_context_new(CString::new("gst.gl.GLDisplay").unwrap().as_ptr(), 1);
                        gst_gl_sys::gst_context_set_gl_display(context, gst_display.as_mut_ptr());
                        gst_gl_sys::gst_element_set_context(src.as_mut_ptr(), context);
                    }*/

                    let context = Context::new("gst.gl.GLDisplay", true);

                    // TODO: add binding to /usr/lib/libgstgl-1.0
                    // (I've got GstGL-1.0.gir for it!)
                    //gst_context_set_gl_display (display_context, sdl_gl_display);
                    unsafe {
                        gst_gl_sys::gst_context_set_gl_display(
                            context.to_glib_none().0,
                            gst_display.as_mut_ptr(),
                        );
                    }
                    src.set_context(&context);
                } else if context_type == "gst.gl.app_context" {
                    //let context = gst_gl_sys::gst_context_new(CString::new("gst.gl.app_context").unwrap().as_ptr(), 1);

                    let mut context = Context::new("gst.gl.app_context", true);
                    {
                        let context_mut = context.make_mut();
                        let structure = context_mut.structure();
                        //structure.set("context", &(gst_context.as_mut_ptr() as i64));
                        unsafe {
                            gstreamer_sys::gst_structure_set(
                                structure.as_mut_ptr(),
                                "context".to_glib_none().0,
                                gst_gl_sys::gst_gl_context_get_type(),
                                gst_context.as_mut_ptr(),
                                0i32,
                            );
                        }
                    }

                    // TODO: check, set doesn't pass value type, may require to bind to 'gst_structure_set' call
                    //structure.set("context", gst_sdl_context);
                    //GstStructure *s = gst_context_writable_structure (app_context);
                    //gst_structure_set (s, "context", gst_gl_context_get_type(),
                    //								   gst_sdl_context,
                    //            NULL);
                    src.set_context(&context);
                }
            }
            _ => (),
        }
    });

    // Build the pipeline
    pipeline
        .add_many(&[
            &source,
            /*&video_convert,*/ &video_sink,
            &audio_convert,
            &audio_sink,
        ])
        .unwrap();
    //video_convert.link(&video_sink).expect("Elements could not be linked.");
    audio_convert
        .link(&audio_sink)
        .expect("Elements could not be linked.");

    // Connect the pad-added signal
    let pipeline_clone = pipeline.clone();
    //let convert_clone = convert.clone();
    //let video_sink_clone = video_convert.clone();
    let video_sink_clone = video_sink.clone();
    let audio_sink_clone = audio_convert.clone();
    source.connect_pad_added(move |_, src_pad| {
        let pipeline = &pipeline_clone;
        let video_sink = &video_sink_clone;
        let audio_sink = &audio_sink_clone;

        println!(
            "Received new pad {} from {}",
            src_pad.name(),
            pipeline.name()
        );

        let new_pad_caps = src_pad
            .current_caps()
            .expect("Failed to get caps of new pad.");
        let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get first structure of caps.");
        let new_pad_type = new_pad_struct.name();

        println!("src pad type: {}", new_pad_type);

        let is_audio = new_pad_type.starts_with("audio/x-raw");
        let is_video = new_pad_type.starts_with("video/x-raw");

        if is_video {
            let sink_pad = video_sink
                .static_pad("sink")
                .expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                println!("We are already linked. Ignoring.");
                return;
            }

            let ret = src_pad.link(&sink_pad);
            if let Ok(_) = ret {
                println!("Type is {} but link failed.", new_pad_type);
            } else {
                println!("Link succeeded (type {}).", new_pad_type);
            }
        }

        if is_audio {
            let sink_pad = audio_sink
                .static_pad("sink")
                .expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                println!("We are already linked. Ignoring.");
                return;
            }

            let ret = src_pad.link(&sink_pad);
            if let Ok(_) = ret {
                println!("Type is {} but link failed.", new_pad_type);
            } else {
                println!("Link succeeded (type {}).", new_pad_type);
            }
        }
    });

    //let video_app_sink = video_sink.dynamic_cast::<gst_app::AppSink>().unwrap();

    (pipeline, video_sink)
}

#[cfg(target_os = "windows")]
pub fn create_opengl_pipeline_url(
    url: &str,
    context: &glutin::Context<PossiblyCurrent>,
) -> (gst::Pipeline, gst::Element) {
    let source = gst::ElementFactory::make("uridecodebin", Some("source"))
        .expect("Could not create uridecodebin element.");
    source.set_property_from_str("uri", url);

    //let video_convert = gst::ElementFactory::make("videoconvert", "videoconvert")
    //  .expect("Could not create videoconvert element.");
    let audio_convert = gst::ElementFactory::make("audioconvert", Some("audioconvert"))
        .expect("Could not create audioconvert element.");

    let video_sink = gst::ElementFactory::make("glimagesink", Some("videosink"))
        .expect("Could not create sink element");
    let audio_sink = gst::ElementFactory::make("autoaudiosink", Some("audiosink"))
        .expect("Could not create sink element.");

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // get display & context handles
    let gl_context = unsafe {
        let context = context.raw_handle();
        if let Wgl(wgl_context) = context {
            wgl_context as usize
        } else {
            unimplemented!()
        }
    };
    //let display = xconnection.display;

    let gst_display = unsafe {
        gst_gl_sys::gst_gl_display_new() /*.as_display()*/
    };

    //let gst_context = unsafe { gst_gl_sys::gst_gl_context_new_wrapped(gst_display.as_mut_ptr(), gl_context as usize,
    //  gst_gl_sys::GST_GL_PLATFORM_WGL, gst_gl_sys::GST_GL_API_OPENGL)  };

    let gst_context = unsafe { gst_gl_sys::gst_gl_context_new(gst_display.as_mut_ptr()) };

    unsafe {
        /*println!("     gl_display: {}, first_bytes: {} {} {} {}", display as usize,
         *((display as usize + 0) as *const usize), *((display as usize + 8) as *const usize),
         *((display as usize + 16) as *const usize), *((display as usize + 24) as *const usize),);*/
        println!(
            "    gst_display: {}, first_bytes: {} {} {} {}",
            gst_display.as_mut_ptr() as usize,
            *((gst_display.as_mut_ptr() as usize + 0) as *const usize),
            *((gst_display.as_mut_ptr() as usize + 8) as *const usize),
            *((gst_display.as_mut_ptr() as usize + 16) as *const usize),
            *((gst_display.as_mut_ptr() as usize + 24) as *const usize),
        );

        println!(
            "     gl_context: {}, first_bytes: {} {} {} {}",
            gl_context as usize,
            *((gl_context as usize + 0) as *const usize),
            *((gl_context as usize + 8) as *const usize),
            *((gl_context as usize + 16) as *const usize),
            *((gl_context as usize + 24) as *const usize),
        );
        println!(
            "     gst_context: {}, first_bytes: {} {} {} {}",
            gst_context.as_mut_ptr() as usize,
            *((gst_context.as_mut_ptr() as usize + 0) as *const usize),
            *((gst_context.as_mut_ptr() as usize + 8) as *const usize),
            *((gst_context.as_mut_ptr() as usize + 16) as *const usize),
            *((gst_context.as_mut_ptr() as usize + 24) as *const usize),
        );
        println!(
            "platform: {}, api: {}",
            gst_gl_sys::gst_gl_context_get_gl_platform(gst_context),
            gst_gl_sys::gst_gl_context_get_gl_api(gst_context)
        );
    }

    let bus = pipeline.get_bus().unwrap();
    bus.add_signal_watch();
    bus.enable_sync_message_emission();
    bus.connect_sync_message(move |_bus, message| {
        //return;
        match message.view() {
            MessageView::NeedContext(need_context) => {
                let context_type = need_context.get_context_type();
                let src = message
                    .get_src()
                    .unwrap()
                    .dynamic_cast::<gst::Element>()
                    .unwrap();
                println!("need context: {:?}, src: {:?}", context_type, src);

                // TODO: add binding for gst_gl_context_new_wrapped from /usr/lib/libgstgl-1.0
                // (I've got GstGL-1.0.gir for it!)
                //
                // sdl_gl_display = gst_gl_display_new ();
                // sdl_gl_display = (GstGLDisplay *) gst_gl_display_x11_new_with_display (sdl_display);
                //
                // gst_sdl_context =
                //      gst_gl_context_new_wrapped (sdl_gl_display, (guintptr) sdl_gl_context,
                //      gst_gl_platform_from_string (platform), GST_GL_API_OPENGL);

                if context_type == "gst.gl.GLDisplay" {
                    /*unsafe {
                        let context = gst_gl_sys::gst_context_new(CString::new("gst.gl.GLDisplay").unwrap().as_ptr(), 1);
                        gst_gl_sys::gst_context_set_gl_display(context, gst_display.as_mut_ptr());
                        gst_gl_sys::gst_element_set_context(src.as_mut_ptr(), context);
                    }*/

                    let context = Context::new("gst.gl.GLDisplay", true);

                    // TODO: add binding to /usr/lib/libgstgl-1.0
                    // (I've got GstGL-1.0.gir for it!)
                    //gst_context_set_gl_display (display_context, sdl_gl_display);
                    unsafe {
                        gst_gl_sys::gst_context_set_gl_display(
                            context.to_glib_none().0,
                            gst_display.as_mut_ptr(),
                        );
                    }
                    src.set_context(&context);
                } else if context_type == "gst.gl.app_context" {
                    //let context = gst_gl_sys::gst_context_new(CString::new("gst.gl.app_context").unwrap().as_ptr(), 1);

                    let mut context = Context::new("gst.gl.app_context", true);
                    {
                        let context_mut = context.make_mut();
                        let structure = context_mut.get_mut_structure();
                        //structure.set("context", &(gst_context.as_mut_ptr() as i64));
                        unsafe {
                            gstreamer_sys::gst_structure_set(
                                structure.as_mut_ptr(),
                                "context".to_glib_none().0,
                                gst_gl_sys::gst_gl_context_get_type(),
                                gst_context.as_mut_ptr(),
                                0i32,
                            );
                        }
                    }

                    // TODO: check, set doesn't pass value type, may require to bind to 'gst_structure_set' call
                    //structure.set("context", gst_sdl_context);
                    //GstStructure *s = gst_context_writable_structure (app_context);
                    //gst_structure_set (s, "context", gst_gl_context_get_type(),
                    //								   gst_sdl_context,
                    //            NULL);

                    // THIS FAILS ON WINDOWS (play doesn't start later)! WHY?
                    //src.set_context(&context);
                }
            }
            _ => (),
        }
    });

    // Build the pipeline
    pipeline
        .add_many(&[
            &source,
            /*&video_convert,*/ &video_sink,
            &audio_convert,
            &audio_sink,
        ])
        .unwrap();
    //video_convert.link(&video_sink).expect("Elements could not be linked.");
    audio_convert
        .link(&audio_sink)
        .expect("Elements could not be linked.");

    // Connect the pad-added signal
    let pipeline_clone = pipeline.clone();
    //let convert_clone = convert.clone();
    //let video_sink_clone = video_convert.clone();
    let video_sink_clone = video_sink.clone();
    let audio_sink_clone = audio_convert.clone();
    source.connect_pad_added(move |_, src_pad| {
        let pipeline = &pipeline_clone;
        let video_sink = &video_sink_clone;
        let audio_sink = &audio_sink_clone;

        println!(
            "Received new pad {} from {}",
            src_pad.get_name(),
            pipeline.get_name()
        );

        let new_pad_caps = src_pad
            .get_current_caps()
            .expect("Failed to get caps of new pad.");
        let new_pad_struct = new_pad_caps
            .get_structure(0)
            .expect("Failed to get first structure of caps.");
        let new_pad_type = new_pad_struct.get_name();

        println!("src pad type: {}", new_pad_type);

        let is_audio = new_pad_type.starts_with("audio/x-raw");
        let is_video = new_pad_type.starts_with("video/x-raw");

        if is_video {
            let sink_pad = video_sink
                .get_static_pad("sink")
                .expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                println!("We are already linked. Ignoring.");
                return;
            }

            let ret = src_pad.link(&sink_pad);
            if let Ok(_) = ret {
                println!("Type is {} but link failed.", new_pad_type);
            } else {
                println!("Link succeeded (type {}).", new_pad_type);
            }
        }

        if is_audio {
            let sink_pad = audio_sink
                .get_static_pad("sink")
                .expect("Failed to get static sink pad from convert");
            if sink_pad.is_linked() {
                println!("We are already linked. Ignoring.");
                return;
            }

            let ret = src_pad.link(&sink_pad);
            if let Ok(_) = ret {
                println!("Type is {} but link failed.", new_pad_type);
            } else {
                println!("Link succeeded (type {}).", new_pad_type);
            }
        }
    });

    //let video_app_sink = video_sink.dynamic_cast::<gst_app::AppSink>().unwrap();

    (pipeline, video_sink)
}
