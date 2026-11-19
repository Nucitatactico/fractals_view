use super::{koch_puntos, triangulo_koch, Fractal, LADO_TRIANGULO_KOCH};
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::dibujo;
use eframe::egui::{self, Pos2, Stroke};
use std::time::Instant;

pub struct CopoKoch {
    pub profundidad: u32,
    pub camara: Camara,
    anim: Animacion,
    /// Contorno completo del copo (polilínea cerrada) en coordenadas del mundo.
    puntos: Vec<Pos2>,
    firma: Option<u32>,
    ms_generacion: f32,
}

impl Default for CopoKoch {
    fn default() -> Self {
        Self {
            profundidad: 4,
            camara: Camara::default(),
            anim: Animacion::new(300.0),
            puntos: Vec::new(),
            firma: None,
            ms_generacion: 0.0,
        }
    }
}

impl CopoKoch {
    /// Versión secuencial clásica: recursión que va empujando puntos al Vec.
    fn generar_lado(a: Pos2, b: Pos2, prof: u32, out: &mut Vec<Pos2>) {
        if prof == 0 {
            out.push(a);
            return;
        }
        let (p1, p2, p3) = koch_puntos(a, b);
        Self::generar_lado(a, p1, prof - 1, out);
        Self::generar_lado(p1, p2, prof - 1, out);
        Self::generar_lado(p2, p3, prof - 1, out);
        Self::generar_lado(p3, b, prof - 1, out);
    }

    fn generar(&mut self) {
        let t0 = Instant::now();
        let [v0, v1, v2] = triangulo_koch();
        let n = 3 * 4usize.pow(self.profundidad);

        self.puntos.clear();
        self.puntos.reserve(n + 1);
        Self::generar_lado(v0, v1, self.profundidad, &mut self.puntos);
        Self::generar_lado(v1, v2, self.profundidad, &mut self.puntos);
        Self::generar_lado(v2, v0, self.profundidad, &mut self.puntos);
        self.puntos.push(v0); // cierra el contorno

        self.firma = Some(self.profundidad);
        self.ms_generacion = t0.elapsed().as_secs_f32() * 1000.0;
    }
}

impl Fractal for CopoKoch {
    fn nombre(&self) -> &'static str {
        "Copo de Koch"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=9)).changed() {
                self.anim.reiniciar();
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 50.0..=50000.0).logarithmic(true));
            if ui.button("Repetir animación").clicked() {
                self.anim.reiniciar();
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
            ui.label(format!(
                "Segmentos: {} | Generación: {:.1} ms",
                self.puntos.len().saturating_sub(1),
                self.ms_generacion
            ));
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

        // LOD según el largo en pantalla de un segmento hoja.
        let largo_px =
            (LADO_TRIANGULO_KOCH / 3f32.powi(self.profundidad as i32) * self.camara.zoom).max(1e-6);
        let paso = ((1.0 / largo_px).ceil() as usize).clamp(1, total.max(1));

        let grosor = (2.0 / (self.profundidad as f32 + 1.0)).max(0.5);
        let color = egui::Color32::from_rgb(100, 200, 255);

        let painter = ui.painter_at(rect);
        dibujo::polilinea(
            &painter,
            &self.camara,
            centro,
            &self.puntos[..=n],
            paso,
            Stroke::new(grosor, color),
        );
    }
}
