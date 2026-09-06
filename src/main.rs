mod state;
mod app;
mod input;
mod model;

fn main() -> anyhow::Result<()> {
    app::App::run()?;
    
    Ok(())
}

