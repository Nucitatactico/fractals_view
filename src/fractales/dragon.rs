use super::Fractal;
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::dibujo;
use eframe::egui::{self, Pos2, Stroke};

/// Longitud de cada segmento en unidades del mundo.
const PASO_MUNDO: f32 = 4.0;

pub struct Dragon {
    pub profundidad: u32,
    pub camara: Camara,
    anim: Animacion,
    /// Vértices de la curva en coordenadas del mundo (se calcula una sola vez).
    puntos: Vec<Pos2>,
    firma: Option<u32>,
}

impl Default for Dragon {
    fn default() -> Self {
        Self {
            profundidad: 12,
            camara: Camara::default(),
            anim: Animacion::new(200.0),
            puntos: Vec::new(),
            firma: None,
        }
    }
}

impl Dragon {
    fn generar(&mut self) {
        let n = 1usize << self.profundidad; // segmentos
        self.puntos.clear();
        self.puntos.reserve(n + 1);

        let mut p = egui::pos2(-100.0, 50.0);
        let mut dir = egui::vec2(0.0, -PASO_MUNDO);
        self.puntos.push(p);
        p += dir;
        self.puntos.push(p);

        for k in 1..n {
            // Secuencia de plegado de papel: sin Vec auxiliar ni trigonometría.
            let positivo = (((k & k.wrapping_neg()) << 1) & k) == 0;
            dir = if positivo {
                egui::vec2(-dir.y, dir.x) // +90°
            } else {
                egui::vec2(dir.y, -dir.x) // -90°
            };
            p += dir;
            self.puntos.push(p);
        }
        self.firma = Some(self.profundidad);
    }
}

impl Fractal for Dragon {
    fn nombre(&self) -> &'static str {
        "Curva del Dragón"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=16)).changed() {
                self.anim.reiniciar();
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 50.0..=20000.0).logarithmic(true));
            if ui.button("Repetir animación").clicked() {
                self.anim.reiniciar();
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let centro = rect.center();
        self.camara.interactuar(ui, rect);

        if self.firma != Some(self.profundidad) {
            self.generar();
        }

        let total = self.puntos.len() - 1;
        let n = self.anim.progreso(ui, total);
        if n == 0 {
            return;
        }

        // LOD: si los segmentos miden < 1 px, salta puntos.
        let largo_px = (PASO_MUNDO * self.camara.zoom).max(1e-6);
        let paso = ((1.0 / largo_px).ceil() as usize).clamp(1, total.max(1));

        let painter = ui.painter_at(rect);
        dibujo::polilinea(
            &painter,
            &self.camara,
            centro,
            &self.puntos[..=n],
            paso,
            Stroke::new(1.5, egui::Color32::from_rgb(255, 200, 100)),
        );
    }
}
