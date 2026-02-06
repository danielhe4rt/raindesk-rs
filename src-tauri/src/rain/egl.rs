use khronos_egl as egl;
use std::ffi::c_void;
use wayland_client::protocol::wl_display::WlDisplay;
use wayland_client::Proxy;

/// EGL state for rendering to a Wayland surface
pub struct EglState {
    pub instance: egl::DynamicInstance<egl::EGL1_4>,
    pub display: egl::Display,
    pub context: egl::Context,
    pub egl_config: egl::Config,
    pub surface: Option<egl::Surface>,
    pub wl_egl_surface: Option<wayland_egl::WlEglSurface>,
}

impl EglState {
    /// Initialize EGL for a Wayland display
    pub fn new(wl_display: &WlDisplay) -> Result<Self, String> {
        // Load libEGL dynamically
        let lib = unsafe { libloading::Library::new("libEGL.so.1") }
            .or_else(|_| unsafe { libloading::Library::new("libEGL.so") })
            .map_err(|e| format!("Failed to load libEGL: {}", e))?;

        let instance = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required_from(lib) }
            .map_err(|e| format!("Failed to create EGL instance: {}", e))?;

        // Get Wayland display pointer
        let wl_display_ptr = wl_display.id().as_ptr() as *mut c_void;

        // Get EGL display for Wayland
        let display = unsafe {
            instance.get_display(wl_display_ptr as egl::NativeDisplayType)
        }
        .ok_or("Failed to get EGL display")?;

        instance
            .initialize(display)
            .map_err(|e| format!("eglInitialize failed: {}", e))?;

        // Bind OpenGL ES API
        instance
            .bind_api(egl::OPENGL_ES_API)
            .map_err(|e| format!("eglBindAPI failed: {}", e))?;

        // Choose config with alpha channel
        let config_attribs = [
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES3_BIT,
            egl::NONE,
        ];

        let egl_config = instance
            .choose_first_config(display, &config_attribs)
            .map_err(|e| format!("eglChooseConfig failed: {}", e))?
            .ok_or("No suitable EGL config found")?;

        // Create OpenGL ES 3.0 context
        let context_attribs = [egl::CONTEXT_MAJOR_VERSION, 3, egl::CONTEXT_MINOR_VERSION, 0, egl::NONE];

        let context = instance
            .create_context(display, egl_config, None, &context_attribs)
            .map_err(|e| format!("eglCreateContext failed: {}", e))?;

        Ok(Self {
            instance,
            display,
            context,
            egl_config,
            surface: None,
            wl_egl_surface: None,
        })
    }

    /// Create an EGL window surface for a Wayland surface
    pub fn create_surface(
        &mut self,
        wl_surface: &wayland_client::protocol::wl_surface::WlSurface,
        width: i32,
        height: i32,
    ) -> Result<(), String> {
        let wl_egl_surface = wayland_egl::WlEglSurface::new(wl_surface.id(), width, height)
            .map_err(|e| format!("Failed to create WlEglSurface: {}", e))?;

        let egl_surface = unsafe {
            self.instance.create_window_surface(
                self.display,
                self.egl_config,
                wl_egl_surface.ptr() as egl::NativeWindowType,
                None,
            )
        }
        .map_err(|e| format!("eglCreateWindowSurface failed: {}", e))?;

        self.surface = Some(egl_surface);
        self.wl_egl_surface = Some(wl_egl_surface);

        Ok(())
    }

    /// Resize the EGL surface
    pub fn resize(&mut self, width: i32, height: i32) {
        if let Some(ref wl_egl_surface) = self.wl_egl_surface {
            wl_egl_surface.resize(width, height, 0, 0);
        }
    }

    /// Make this context current
    pub fn make_current(&self) -> Result<(), String> {
        self.instance
            .make_current(self.display, self.surface, self.surface, Some(self.context))
            .map_err(|e| format!("eglMakeCurrent failed: {}", e))
    }

    /// Swap buffers
    pub fn swap_buffers(&self) -> Result<(), String> {
        if let Some(surface) = self.surface {
            self.instance
                .swap_buffers(self.display, surface)
                .map_err(|e| format!("eglSwapBuffers failed: {}", e))
        } else {
            Err("No EGL surface".to_string())
        }
    }

    /// Create a glow context from EGL
    pub fn create_gl_context(&self) -> glow::Context {
        unsafe {
            glow::Context::from_loader_function_cstr(|name| {
                let name_str = name.to_str().unwrap_or("");
                self.instance
                    .get_proc_address(name_str)
                    .map_or(std::ptr::null(), |p| p as *const _)
            })
        }
    }
}

impl Drop for EglState {
    fn drop(&mut self) {
        if let Some(surface) = self.surface.take() {
            let _ = self.instance.destroy_surface(self.display, surface);
        }
        let _ = self
            .instance
            .destroy_context(self.display, self.context);
        let _ = self.instance.terminate(self.display);
    }
}
