mod animacion;
mod camara;
mod curva;
mod dibujo;
mod fractales;
mod hilos;
mod pixel_art;

use pixel_art::MyPixelArt;

fn main() -> eframe::Result<()> {
    let title = String::from("Fractales");
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
