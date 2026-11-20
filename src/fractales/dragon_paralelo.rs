use super::dragon::PASO_MUNDO;
use super::Fractal;
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::curva::Curva;
use crate::hilos;
use eframe::egui::{self, Pos2, Vec2};
use rayon::prelude::*;

/// Dirección del segmento `k` en forma cerrada: el rumbo (en múltiplos de 90°)
/// es el número de unos del código Gray de k. Por eso cada bloque puede calcularse
/// sin conocer los anteriores.
fn direccion(k: usize) -> Vec2 {
    match (k ^ (k >> 1)).count_ones() % 4 {
        0 => egui::vec2(0.0, -PASO_MUNDO),
        1 => egui::vec2(PASO_MUNDO, 0.0),
        2 => egui::vec2(0.0, PASO_MUNDO),
        _ => egui::vec2(-PASO_MUNDO, 0.0),
    }
}

pub struct DragonParalelo {
    pub profundidad: u32,
    pub camara: Camara,
    anim: Animacion,
    curva: Curva,
    trabajadores: usize,
    firma: Option<u32>,
}

impl Default for DragonParalelo {
    fn default() -> Self {
        Self {
            profundidad: 12,
            camara: Camara::default(),
            anim: Animacion::new(200.0),
            curva: Curva::vacia(),
            trabajadores: hilos::trabajadores(),
            firma: None,
        }
    }
}

impl DragonParalelo {
    /// Suma de prefijos (scan) paralela en 3 fases:
    ///  1. (paralela)   cada bloque suma sus desplazamientos
    ///  2. (secuencial) prefijos sobre las pocas sumas de bloque
    ///  3. (paralela)   cada bloque escribe sus puntos partiendo de su prefijo
    fn generar(&mut self) {
        let w = hilos::trabajadores();
        let n = 1usize << self.profundidad;
        let tam = ((n + w - 1) / w).max(1);
        let origen = egui::vec2(-100.0, 50.0);

        let sumas: Vec<Vec2> = (0..w)
            .into_par_iter()
            .map(|b| {
                let ini = (b * tam).min(n);
                let fin = ((b + 1) * tam).min(n);
                let mut s = Vec2::ZERO;
                for k in ini..fin {
                    s += direccion(k);
                }
                s
            })
            .collect();

        let mut inicios: Vec<Vec2> = Vec::with_capacity(w);
        let mut acc = origen;
        for s in &sumas {
            inicios.push(acc);
            acc += *s;
        }

        let mut puntos = vec![Pos2::ZERO; n + 1];
        puntos[0] = origen.to_pos2();
        puntos[1..]
            .par_chunks_mut(tam)
            .enumerate()
            .for_each(|(b, trozo)| {
                let ini = b * tam;
                let mut p = inicios[b].to_pos2();
                for (j, slot) in trozo.iter_mut().enumerate() {
                    p += direccion(ini + j);
                    *slot = p;
                }
            });

        self.trabajadores = w;
        self.curva = Curva::bloques(puntos, w);
        self.firma = Some(self.profundidad);
    }
}

impl Fractal for DragonParalelo {
    fn nombre(&self) -> &'static str {
        "Curva del Dragón (Paralelo)"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=18)).changed() {
                self.anim.reiniciar();
            }
            ui.label("Velocidad (por hilo):");
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

        let painter = ui.painter_at(rect);
        self.curva.dibujar(
            &painter,
            &self.camara,
            centro,
            n,
            self.anim.animando,
            paso,
            1.5,
            hilos::color_hilo,
        );
        let titulo = format!("Paralelo: {} hilos", self.trabajadores);
        hilos::panel(&painter, rect, &titulo, &self.curva.filas(n));
    }
}
