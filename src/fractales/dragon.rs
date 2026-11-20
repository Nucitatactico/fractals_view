use super::Fractal;
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::curva::Curva;
use crate::hilos;
use eframe::egui::{self, Pos2};

/// Longitud de cada segmento en unidades del mundo.
pub const PASO_MUNDO: f32 = 4.0;

pub struct Dragon {
    pub profundidad: u32,
    pub camara: Camara,
    anim: Animacion,
    curva: Curva,
    firma: Option<u32>,
}

impl Default for Dragon {
    fn default() -> Self {
        Self {
            profundidad: 12,
            camara: Camara::default(),
            anim: Animacion::new(200.0),
            curva: Curva::vacia(),
            firma: None,
        }
    }
}

impl Dragon {
    /// Secuencial: un único recorrido que gira según la secuencia de plegado de papel.
    fn generar(&mut self) {
        let n = 1usize << self.profundidad; // segmentos
        let mut puntos: Vec<Pos2> = Vec::with_capacity(n + 1);

        let mut p = egui::pos2(-100.0, 50.0);
        let mut dir = egui::vec2(0.0, -PASO_MUNDO);
        puntos.push(p);
        p += dir;
        puntos.push(p);

        for k in 1..n {
            let positivo = (((k & k.wrapping_neg()) << 1) & k) == 0;
            dir = if positivo {
                egui::vec2(-dir.y, dir.x) // +90°
            } else {
                egui::vec2(dir.y, -dir.x) // -90°
            };
            p += dir;
            puntos.push(p);
        }

        self.curva = Curva::secuencial(puntos);
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
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=18)).changed() {
                self.anim.reiniciar();
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 50.0..=200000.0).logarithmic(true));
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

        let total = self.curva.max_trabajo;
        let n = self.anim.progreso(ui, total);

        let largo_px = (PASO_MUNDO * self.camara.zoom).max(1e-6);
        let paso = ((1.0 / largo_px).ceil() as usize).clamp(1, total.max(1));
        let color = egui::Color32::from_rgb(255, 200, 100);

        let painter = ui.painter_at(rect);
        self.curva.dibujar(
            &painter,
            &self.camara,
            centro,
            n,
            self.anim.animando,
            paso,
            1.5,
            |_| color,
        );
        hilos::panel(&painter, rect, "Secuencial: 1 hilo", &self.curva.filas(n));
    }
}
