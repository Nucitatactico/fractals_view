pub mod arbol_pitagoras;
pub mod copo_koch;
pub mod romanesco;
pub mod apolonio;   // <-- NUEVO
pub mod dragon;     // <-- NUEVO

use eframe::egui;

pub trait Fractal {
    fn nombre(&self) -> &'static str;
    fn ui_controles(&mut self, ui: &mut egui::Ui);
    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect);
}
