mod animacion;
mod camara;
mod curva;
mod dibujo;
mod escena;
mod fractales;
mod hilos;
mod pixel_art;
mod bench;

use pixel_art::MyPixelArt;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--bench") {
        bench::ejecutar(&args);
        return Ok(());
    }
    let title = String::from("Fractales EDA2");
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 1000.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title(&title),
        ..Default::default()
    };
    eframe::run_native(
        &title,
        native_options,
        Box::new(|cc| Ok(Box::new(MyPixelArt::new(cc)))),
    )
}
