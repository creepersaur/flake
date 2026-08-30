mod state;
mod app;

fn main() -> anyhow::Result<()> {
    app::App::run()?;
    
    Ok(())
}

