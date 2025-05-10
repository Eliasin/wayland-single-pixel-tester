/*
 * Adapted from the "simple_window.rs" example of wayland-client.
 */

use wayland_client::{
    delegate_noop,
    protocol::{
        wl_buffer::{self, WlBuffer},
        wl_compositor, wl_keyboard, wl_registry, wl_seat, wl_shm_pool,
        wl_surface::{self, WlSurface},
    },
    Connection, Dispatch, QueueHandle, WEnum,
};

use wayland_protocols::wp::viewporter::client::{
    wp_viewport::WpViewport, wp_viewporter::WpViewporter,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_protocols::{
    wp::single_pixel_buffer::v1::client::wp_single_pixel_buffer_manager_v1::WpSinglePixelBufferManagerV1,
    xdg::shell::client::xdg_wm_base::XdgWmBase,
};

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let mut event_queue = conn.new_event_queue();
    let qhandle = event_queue.handle();

    let display = conn.display();
    display.get_registry(&qhandle, ());

    let mut state = State::Uninitialized {
        compositor: None,
        single_pixel_buffer_manager: None,
        viewporter: None,
        seat: None,
        xdg_wm_base: None,
        qh: qhandle,
    };
    println!("Starting the example window app, press <ESC> to quit.");

    let mut running = true;
    while running {
        event_queue.blocking_dispatch(&mut state).unwrap();

        if let State::Uninitialized { .. } = state {
            running = true;
        }
        if let State::Initialized {
            running: state_running,
            ..
        } = state
        {
            running = state_running;
        }
    }
}

enum State {
    Uninitialized {
        compositor: Option<wl_compositor::WlCompositor>,
        single_pixel_buffer_manager: Option<WpSinglePixelBufferManagerV1>,
        viewporter: Option<WpViewporter>,
        seat: Option<wl_seat::WlSeat>,
        xdg_wm_base: Option<XdgWmBase>,
        qh: QueueHandle<State>,
    },
    Initialized {
        compositor: wl_compositor::WlCompositor,
        single_pixel_buffer_manager: WpSinglePixelBufferManagerV1,
        viewporter: WpViewporter,
        viewport: Option<WpViewport>,
        seat: wl_seat::WlSeat,
        xdg_wm_base: XdgWmBase,
        qh: QueueHandle<State>,
        buffer: WlBuffer,
        base_surface: WlSurface,
        running: bool,
    },
}

impl State {
    fn try_initialize(&mut self) {
        match self {
            Self::Uninitialized {
                compositor: Some(compositor),
                single_pixel_buffer_manager: Some(single_pixel_buffer_manager),
                viewporter: Some(viewporter),
                seat: Some(seat),
                xdg_wm_base: Some(xdg_wm_base),
                qh,
            } => {
                let surface = compositor.create_surface(qh, ());
                let xdg_surface = xdg_wm_base.get_xdg_surface(&surface, qh, ());
                let toplevel = xdg_surface.get_toplevel(qh, ());
                toplevel.set_title("My test :^)".into());

                surface.commit();

                let buffer = single_pixel_buffer_manager.create_u32_rgba_buffer(
                    u32::MAX / 2,
                    0,
                    0,
                    u32::MAX / 2,
                    qh,
                    (),
                );

                *self = Self::Initialized {
                    base_surface: surface,
                    compositor: compositor.clone(),
                    single_pixel_buffer_manager: single_pixel_buffer_manager.clone(),
                    viewporter: viewporter.clone(),
                    viewport: None,
                    seat: seat.clone(),
                    xdg_wm_base: xdg_wm_base.clone(),
                    qh: qh.clone(),
                    buffer,
                    running: true,
                };
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());

                    if let State::Uninitialized { compositor: c, .. } = state {
                        *c = Some(compositor);
                    }
                    state.try_initialize();
                }
                "wp_single_pixel_buffer_manager_v1" => {
                    let single_pixel_buffer_manager =
                        registry.bind::<WpSinglePixelBufferManagerV1, _, _>(name, 1, qh, ());

                    if let State::Uninitialized {
                        single_pixel_buffer_manager: s,
                        ..
                    } = state
                    {
                        *s = Some(single_pixel_buffer_manager);
                    }

                    state.try_initialize();
                }
                "wp_viewporter" => {
                    let viewporter = registry.bind::<WpViewporter, _, _>(name, 1, qh, ());

                    if let State::Uninitialized { viewporter: v, .. } = state {
                        *v = Some(viewporter);
                    }
                    state.try_initialize();
                }
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());

                    if let State::Uninitialized { seat: s, .. } = state {
                        *s = Some(seat);
                    }
                    state.try_initialize();
                }
                "xdg_wm_base" => {
                    let wm_base = registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, 1, qh, ());
                    if let State::Uninitialized { xdg_wm_base: x, .. } = state {
                        *x = Some(wm_base);
                    }

                    state.try_initialize();
                }
                _ => {}
            }
        }
    }
}

// Ignore events from these object types in this example.
delegate_noop!(State: ignore WpSinglePixelBufferManagerV1);
delegate_noop!(State: ignore WpViewporter);
delegate_noop!(State: ignore WpViewport);
delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let State::Initialized {
            base_surface,
            buffer,
            viewporter,
            viewport,
            ..
        } = state
        {
            if let xdg_surface::Event::Configure { serial, .. } = event {
                xdg_surface.ack_configure(serial);

                if viewport.is_none() {
                    let v = viewporter.get_viewport(base_surface, qh, ());
                    v.set_source(0.0, 0.0, 1.0, 1.0);
                    v.set_destination(640, 640);
                    *viewport = Some(v);
                }

                base_surface.attach(Some(buffer), 0, 0);
                base_surface.commit();
            }
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Close = event {
            if let State::Initialized { running, .. } = state {
                *running = false;
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                // ESC key
                if let State::Initialized { running, .. } = state {
                    *running = false;
                }
            }
        }
    }
}
