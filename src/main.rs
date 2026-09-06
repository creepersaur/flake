mod state;
mod app;
mod input;
mod model;
mod camera;
mod draw_state;
mod shapes;

fn main() -> anyhow::Result<()> {
    app::App::run()?;
    
    Ok(())
}


