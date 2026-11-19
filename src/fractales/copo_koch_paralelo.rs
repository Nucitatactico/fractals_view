use super::{koch_puntos, triangulo_koch, Fractal, LADO_TRIANGULO_KOCH};
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::dibujo;
use eframe::egui::{self, Pos2, Stroke};
use rayon::prelude::*;
use std::time::Instant;

/// Por debajo de este nivel restante se calcula en secuencial
/// (paralelizar tareas diminutas cuesta más que el trabajo útil).
const UMBRAL_PARALELO: u32 = 6;

/// Rellena `out` (longitud exacta 4^prof) con el punto de inicio de cada segmento hoja.
/// Sin Vec intermedios: cada tarea escribe directo en su porción del slice.
fn rellenar(a: Pos2, b: Pos2, prof: u32, out: &mut [Pos2]) {
    if prof == 0 {
        out[0] = a;
        return;
    }
    let (p1, p2, p3) = koch_puntos(a, b);
    let tareas = [(a, p1), (p1, p2), (p2, p3), (p3, b)];
    let q = out.len() / 4;

    if prof > UMBRAL_PARALELO {
        out.par_chunks_mut(q)
            .zip(tareas.as_slice().par_iter())
            .for_each(|(trozo, &(s, e))| rellenar(s, e, prof - 1, trozo));
    } else {
        for (trozo, &(s, e)) in out.chunks_mut(q).zip(tareas.iter()) {
            rellenar(s, e, prof - 1, trozo);
        }
    }
}

fn color_hilo(hilo: usize, prof: u32) -> egui::Color32 {
    let base_hue = match hilo {
        0 => 0.55, // cian
        1 => 0.05, // naranja
        _ => 0.30, // verde
    };
    let hue = (base_hue + prof as f32 * 0.02).rem_euclid(1.0);
    let val = 0.9 - (prof as f32 * 0.02).min(0.3);
    egui::Color32::from(egui::ecolor::Hsva::new(hue, 0.8, val, 1.0))
}

pub struct CopoKochParalelo {
    pub profundidad: u32,
    pub camara: Camara,
    anim: Animacion,
    /// Un contorno por lado del triángulo (cada uno con 4^prof + 1 puntos).
    lados: [Vec<Pos2>; 3],
    firma: Option<u32>,
    ms_generacion: f32,
}

impl Default for CopoKochParalelo {
    fn default() -> Self {
        Self {
            profundidad: 5,
            camara: Camara::default(),
            anim: Animacion::new(200.0),
            lados: Default::default(),
            firma: None,
            ms_generacion: 0.0,
        }
    }
}

impl CopoKochParalelo {
    fn generar(&mut self) {
        let t0 = Instant::now();
        let vertices = triangulo_koch();
        let prof = self.profundidad;
        let n = 4usize.pow(prof);

        // Un lado por tarea de rayon; dentro de cada lado, `rellenar` se subdivide en paralelo.
        self.lados[..]
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, v)| {
                let a = vertices[i];
                let b = vertices[(i + 1) % 3];
                v.clear();
                v.resize(n + 1, Pos2::ZERO);
                let (cuerpo, cola) = v.split_at_mut(n);
                rellenar(a, b, prof, cuerpo);
                cola[0] = b;
            });

        self.firma = Some(prof);
        self.ms_generacion = t0.elapsed().as_secs_f32() * 1000.0;
    }
}

impl Fractal for CopoKochParalelo {
    fn nombre(&self) -> &'static str {
        "Copo de Koch (Paralelo)"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=10)).changed() {
                self.anim.reiniciar();
            }
            ui.label("Velocidad (por hilo):");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 50.0..=50000.0).logarithmic(true));
            if ui.button("Repetir animación").clicked() {
                self.anim.reiniciar();
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
            ui.label(format!(
                "Segmentos: {} | Generación: {:.1} ms",
                3 * 4usize.pow(self.profundidad),
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

        // Los 3 "hilos" avanzan a la vez: cada uno revela su propio lado.
        let total_por_lado = self.lados[0].len() - 1;
        let n = self.anim.progreso(ui, total_por_lado);
        if n == 0 {
            return;
        }

        let largo_px =
            (LADO_TRIANGULO_KOCH / 3f32.powi(self.profundidad as i32) * self.camara.zoom).max(1e-6);
        let paso = ((1.0 / largo_px).ceil() as usize).clamp(1, total_por_lado.max(1));
        let grosor = (2.0 / (self.profundidad as f32 + 1.0)).max(0.5);

        let painter = ui.painter_at(rect);
        for (hilo, lado) in self.lados.iter().enumerate() {
            dibujo::polilinea(
                &painter,
                &self.camara,
                centro,
                &lado[..=n],
                paso,
                Stroke::new(grosor, color_hilo(hilo, self.profundidad)),
            );
        }
    }
}
