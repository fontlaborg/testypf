use std::env;
use std::path::PathBuf;

use testypf_core::{FontliftFontSource, RenderSettings, TestypfEngine};

fn usage() -> ! {
    eprintln!("Usage: cargo run --example render_once -- <font-path> [sample text]");
    std::process::exit(1);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let font_path = args.next().map(PathBuf::from).unwrap_or_else(|| usage());
    let sample_text = args
        .next()
        .unwrap_or_else(|| "The quick brown fox jumps over the lazy dog".to_string());

    let mut engine = TestypfEngine::new()?;
    let source = FontliftFontSource::new(font_path);
    let font = engine.font_manager().add_font(&source)?;

    let settings = RenderSettings {
        sample_text,
        ..RenderSettings::default()
    };

    let mut results = engine.render_previews(&settings)?;
    let (_, render) = results
        .pop()
        .ok_or("render_previews returned no results; check typf setup")?;

    println!("Font: {}", font.full_name);
    println!("Backend: {}", settings.backend);
    println!("Dimensions: {}x{}", render.width, render.height);
    println!("Pixel data: {} bytes (RGBA)", render.data.len());

    Ok(())
}
