use crate::global::PENDING_CONFIG;
use crate::state::State;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Icon, Window, WindowId};

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

    fn set_state_prop<T>(&mut self, value: Option<T>, f: impl Fn(&T, &mut State)) {
        value
            .as_ref()
            .and_then(|a| Some(f(a, self.state.as_mut().unwrap())));
    }
}

impl<F: FnMut()> ApplicationHandler<State> for App<F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let cfg = PENDING_CONFIG.with_borrow(|s| s.clone());

        let default_icon = || {
            let img = image::load_from_memory(include_bytes!("flake_icon.png"))
                .unwrap()
                .into_rgba8();
            let (w, h) = img.dimensions();
            (img.into_raw(), w, h)
        };

        let (icon_data, icon_w, icon_h) = cfg.window_icon.unwrap_or_else(default_icon);
        let icon = Icon::from_rgba(icon_data, icon_w, icon_h).ok();

        let window_attributes = Window::default_attributes()
            .with_title(cfg.window_title.unwrap_or("Flake App".into()))
            .with_decorations(cfg.window_decorations.unwrap_or(true))
            .with_window_icon(icon)
            .with_inner_size(PhysicalSize::new(
                cfg.window_width.unwrap_or(800),
                cfg.window_height.unwrap_or(600),
            ))
            .with_window_level(cfg.window_level.unwrap_or_default())
            .with_visible(false);

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.state = Some(pollster::block_on(State::new(window.clone())).unwrap());

        PENDING_CONFIG.with_borrow(|s| {
            self.set_state_prop(s.window_x, |value, state| state.set_window_x(*value));
            self.set_state_prop(s.window_y, |value, state| state.set_window_y(*value));
            self.set_state_prop(s.window_passthrough, |value, state| {
                state.set_window_passthrough(*value)
            });

            self.set_state_prop(s.fps_capped, |value, state| state.set_fps_capped(*value));
            self.set_state_prop(s.target_fps, |value, state| state.set_target_fps(*value));

            self.state
                .as_mut()
                .unwrap()
                .load_font_from_bytes(include_bytes!("Jetbrains.ttf"));
            s.fonts.iter().for_each(|x| self.state.as_mut().unwrap().load_font(x.clone()));

            s.timers
                .iter()
                .for_each(|x| self.state.as_mut().unwrap().add_timer(x.clone()))
        });

        window.set_visible(true);
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
