mod camara;
mod fractales;
mod pixel_art;

use pixel_art::MyPixelArt;

fn main() {
    let title = String::from("Fractales");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        &title,
        native_options,
        Box::new(|cc| Ok(Box::new(MyPixelArt::new(cc)))),
    )
    .unwrap();
}
