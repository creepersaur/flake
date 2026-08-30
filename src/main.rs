mod state;
mod app;
mod vertex;
mod texture;

fn main() -> anyhow::Result<()> {
    app::App::run()?;
    
    Ok(())
}

