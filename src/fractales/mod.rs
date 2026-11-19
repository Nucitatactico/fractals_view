pub mod arbol_pitagoras;
pub mod arbol_pitagoras_paralelo;
pub mod copo_koch;
pub mod copo_koch_paralelo;
pub mod dragon;
pub mod romanesco;

use eframe::egui;

pub trait Fractal {
    fn nombre(&self) -> &'static str;
    fn ui_controles(&mut self, ui: &mut egui::Ui);
    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect);
}

// ---------- Utilidades compartidas por los dos copos de Koch ----------

pub const LADO_TRIANGULO_KOCH: f32 = 200.0;

pub const ANGULO_KOCH: f32 = -std::f32::consts::FRAC_PI_3;

pub fn triangulo_koch() -> [egui::Pos2; 3] {
    let r = LADO_TRIANGULO_KOCH / 3.0_f32.sqrt();
    let mitad = LADO_TRIANGULO_KOCH / 2.0;
    [
        egui::pos2(0.0, -r),
        egui::pos2(mitad, r * 0.5),
        egui::pos2(-mitad, r * 0.5),
    ]
}

pub fn koch_puntos(a: egui::Pos2, b: egui::Pos2) -> (egui::Pos2, egui::Pos2, egui::Pos2) {
    let v = b - a;
    let p1 = a + v / 3.0;
    let p3 = a + v * (2.0 / 3.0);
    let (sin, cos) = ANGULO_KOCH.sin_cos();
    let t = v / 3.0;
    let p2 = p1 + egui::vec2(t.x * cos - t.y * sin, t.x * sin + t.y * cos);
    (p1, p2, p3)
}
