use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{wl_compositor, wl_output, wl_region, wl_registry, wl_surface};
use wayland_client::{delegate_noop, Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1, zwlr_layer_surface_v1,
};

use crate::config::RainConfig;
use crate::rain::egl::EglState;
use crate::rain::particles::ParticleSystem;
use crate::rain::renderer::Renderer;
use crate::state::OverlaySignal;

/// Runs the rain overlay on the current thread (blocking).
/// Call from a dedicated `std::thread::spawn`.
pub fn run_overlay(config: Arc<Mutex<RainConfig>>, rx: mpsc::Receiver<OverlaySignal>) {
    if let Err(e) = run_overlay_inner(config, rx) {
        eprintln!("[raindesk overlay] Error: {}", e);
    }
}

fn run_overlay_inner(
    config: Arc<Mutex<RainConfig>>,
    rx: mpsc::Receiver<OverlaySignal>,
) -> Result<(), String> {
    // Connect to Wayland
    eprintln!("[raindesk overlay] Connecting to Wayland...");
    let conn = Connection::connect_to_env().map_err(|e| format!("Wayland connect: {}", e))?;
    eprintln!("[raindesk overlay] Connected to Wayland");
    let (globals, mut event_queue) =
        registry_queue_init::<OverlayState>(&conn).map_err(|e| format!("Registry init: {}", e))?;
    let qh = event_queue.handle();

    // Bind globals
    let compositor: wl_compositor::WlCompositor = globals
        .bind(&qh, 4..=6, ())
        .map_err(|e| format!("wl_compositor: {}", e))?;
    let layer_shell: zwlr_layer_shell_v1::ZwlrLayerShellV1 = globals
        .bind(&qh, 1..=4, ())
        .map_err(|e| format!("zwlr_layer_shell_v1: {}", e))?;

    // Create surface
    let wl_surface = compositor.create_surface(&qh, ());

    // Create layer surface (Overlay layer, all edges anchored, fullscreen)
    let layer_surface = layer_shell.get_layer_surface(
        &wl_surface,
        None, // default output
        zwlr_layer_shell_v1::Layer::Overlay,
        "raindesk".to_string(),
        &qh,
        (),
    );

    // Configure: anchor all edges (fullscreen), exclusive zone -1 (don't reserve space)
    layer_surface.set_anchor(
        zwlr_layer_surface_v1::Anchor::Top
            | zwlr_layer_surface_v1::Anchor::Bottom
            | zwlr_layer_surface_v1::Anchor::Left
            | zwlr_layer_surface_v1::Anchor::Right,
    );
    layer_surface.set_exclusive_zone(-1);
    layer_surface.set_keyboard_interactivity(
        zwlr_layer_surface_v1::KeyboardInteractivity::None,
    );

    // Set empty input region (click-through)
    let region: wl_region::WlRegion = compositor.create_region(&qh, ());
    wl_surface.set_input_region(Some(&region));
    region.destroy();

    // Initial commit to get configure event
    wl_surface.commit();

    // State
    let mut state = OverlayState {
        configured: false,
        width: 0,
        height: 0,
        closed: false,
    };

    // Process events until we get a configure
    while !state.configured && !state.closed {
        event_queue
            .blocking_dispatch(&mut state)
            .map_err(|e| format!("Dispatch: {}", e))?;
    }

    if state.closed || state.width == 0 || state.height == 0 {
        return Err("Layer surface closed or zero size".to_string());
    }

    let mut w = state.width as i32;
    let mut h = state.height as i32;
    eprintln!("[raindesk overlay] Configured: {}x{}", w, h);

    // Initialize EGL
    eprintln!("[raindesk overlay] Initializing EGL...");
    let wl_display = conn.display();
    let mut egl = EglState::new(&wl_display)?;
    egl.create_surface(&wl_surface, w, h)?;
    egl.make_current()?;
    eprintln!("[raindesk overlay] EGL initialized");

    // Create GL context and renderer
    let gl = egl.create_gl_context();
    let mut renderer = Renderer::new(gl, w as f32, h as f32)?;
    eprintln!("[raindesk overlay] Renderer ready");

    // Create particle system
    let initial_config = config.lock().unwrap().clone();
    let mut particles = ParticleSystem::new(w as f32, h as f32, &initial_config);

    // Frame loop
    eprintln!("[raindesk overlay] Entering frame loop");
    let mut last_frame = Instant::now();
    let frame_target = std::time::Duration::from_micros(16_667); // ~60fps

    loop {
        // Check for shutdown/config signals (non-blocking)
        loop {
            match rx.try_recv() {
                Ok(OverlaySignal::Shutdown) => return Ok(()),
                Ok(OverlaySignal::ConfigChanged) => {
                    let cfg = config.lock().unwrap().clone();
                    particles.update_config(&cfg);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
            }
        }

        // Dispatch any pending Wayland events (non-blocking)
        event_queue
            .dispatch_pending(&mut state)
            .map_err(|e| format!("Dispatch: {}", e))?;

        // Flush the display to send any pending requests
        let _ = conn.flush();

        // Read any events from the Wayland socket (non-blocking)
        if let Some(guard) = conn.prepare_read() {
            let _ = guard.read();
        }
        event_queue
            .dispatch_pending(&mut state)
            .map_err(|e| format!("Dispatch: {}", e))?;

        if state.closed {
            return Ok(());
        }

        // Handle resize
        if state.width as i32 != w || state.height as i32 != h {
            w = state.width as i32;
            h = state.height as i32;
            egl.resize(w, h);
            renderer.resize(w as f32, h as f32);
            particles.resize(w as f32, h as f32);
        }

        // Calculate delta time
        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        // Update particles
        particles.update(dt);

        // Render
        egl.make_current()?;
        renderer.render(&particles);
        egl.swap_buffers()?;
        let _ = conn.flush();

        // Frame pacing — sleep for remainder of frame budget
        let elapsed = now.elapsed();
        if elapsed < frame_target {
            std::thread::sleep(frame_target - elapsed);
        }
    }
}

/// Minimal state for the overlay Wayland client
struct OverlayState {
    configured: bool,
    width: u32,
    height: u32,
    closed: bool,
}

// Dispatch for layer surface events
impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for OverlayState {
    fn event(
        state: &mut Self,
        surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                surface.ack_configure(serial);
                if width > 0 && height > 0 {
                    state.width = width;
                    state.height = height;
                }
                state.configured = true;
            }
            zwlr_layer_surface_v1::Event::Closed => {
                state.closed = true;
            }
            _ => {}
        }
    }
}

// Dispatch for layer shell (no events to handle)
impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for OverlayState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _event: zwlr_layer_shell_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

// No-op dispatches for protocols we need but don't handle events for
delegate_noop!(OverlayState: ignore wl_compositor::WlCompositor);
delegate_noop!(OverlayState: ignore wl_surface::WlSurface);
delegate_noop!(OverlayState: ignore wl_region::WlRegion);
delegate_noop!(OverlayState: ignore wl_output::WlOutput);

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for OverlayState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}
