pub mod arbol_pitagoras;
pub mod arbol_pitagoras_paralelo;
pub mod copo_koch;
pub mod copo_koch_paralelo;
pub mod dragon;
pub mod dragon_paralelo;
pub mod mandelbrot;
pub mod romanesco;
pub mod romanesco_paralelo;

use eframe::egui;

pub trait Fractal {
    fn nombre(&self) -> &'static str;
    fn ui_controles(&mut self, ui: &mut egui::Ui);
    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect);
}

// ---------- Utilidades compartidas por los dos copos de Koch ----------

pub const LADO_TRIANGULO_KOCH: f32 = 200.0;

/// Son constantes para no llamar a `sin_cos` millones de veces en profundidades altas.
const COS_KOCH: f32 = 0.5;
const SEN_KOCH: f32 = -0.866_025_4;

/// Triángulo equilátero centrado en el origen (coordenadas del mundo).
pub fn triangulo_koch() -> [egui::Pos2; 3] {
    let r = LADO_TRIANGULO_KOCH / 3.0_f32.sqrt();
    let mitad = LADO_TRIANGULO_KOCH / 2.0;
    [
        egui::pos2(0.0, -r),
        egui::pos2(mitad, r * 0.5),
        egui::pos2(-mitad, r * 0.5),
    ]
}

/// Dado un segmento a-b devuelve p1, p2 (la punta) y p3 de la curva de Koch.
pub fn koch_puntos(a: egui::Pos2, b: egui::Pos2) -> (egui::Pos2, egui::Pos2, egui::Pos2) {
    let t = (b - a) / 3.0;
    let p1 = a + t;
    let p3 = a + t * 2.0;
    let p2 = p1 + egui::vec2(t.x * COS_KOCH - t.y * SEN_KOCH, t.x * SEN_KOCH + t.y * COS_KOCH);
    (p1, p2, p3)
}
