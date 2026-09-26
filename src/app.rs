use crate::global::PENDING_CONFIG;
use crate::state::State;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

pub struct App<F: FnMut()> {
    pub state: Option<State>,
    update_fn: F,
}

impl<F: FnMut()> App<F> {
    pub fn run(update_fn: F) -> anyhow::Result<()> {
        env_logger::init();
        let event_loop = EventLoop::with_user_event().build()?;

        let mut app = App {
            state: None,
            update_fn,
        };
        event_loop.run_app(&mut app)?;

        Ok(())
    }
}

impl<F: FnMut()> ApplicationHandler<State> for App<F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes().with_visible(false);
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        PENDING_CONFIG.with_borrow(|s| {
            s.window_title
                .as_ref()
                .and_then(|title| Some(window.set_title(title)));

            if let Some((rgba, w, h)) = &s.window_icon {
                if let Ok(icon) = winit::window::Icon::from_rgba(rgba.clone(), *w, *h) {
                    window.set_window_icon(Some(icon));
                }
            }

            s.window_x.as_ref().and_then(|x| {
                let pos = window.outer_position().unwrap_or_default();
                Some(window.set_outer_position(PhysicalPosition::new(*x, pos.y)))
            });

            s.window_y.as_ref().and_then(|y| {
                let pos = window.outer_position().unwrap_or_default();
                Some(window.set_outer_position(PhysicalPosition::new(pos.x, *y)))
            });

            s.window_width.as_ref().and_then(|w| {
                let size = window.outer_size();
                Some(window.request_inner_size(PhysicalSize::new(*w, size.height)))
            });

            s.window_height.as_ref().and_then(|h| {
                let size = window.outer_size();
                Some(window.request_inner_size(PhysicalSize::new(size.width, *h)))
            });
        });

        window.set_visible(true);
        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: State) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if !state.running {
                    event_loop.exit()
                }

                state.update(&mut self.update_fn);

                match state.render() {
                    Ok(_) => {}
                    Err(e) => {
                        // Log the error and exit gracefully
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if !repeat {
                    state.handle_key(event_loop, code, key_state.is_pressed())
                }
            }
            WindowEvent::MouseInput {
                button,
                state: element_state,
                ..
            } => state.handle_mouse_button(button, element_state.is_pressed()),
            WindowEvent::CursorMoved { position, .. } => {
                state.handle_mouse_motion(position.x as f32, position.y as f32)
            }

            _ => {}
        }
    }
}
