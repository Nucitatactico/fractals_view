use super::Fractal;
use crate::curva::Curva;
use crate::escena::Escena;
use crate::hilos;
use eframe::egui::{self, Color32, Pos2, Vec2};
use rayon::prelude::*;

/// Longitud de cada segmento en unidades del mundo.
const PASO_MUNDO: f32 = 4.0;
const ORIGEN: Vec2 = Vec2::new(-100.0, 50.0);

/// Dirección del segmento `k` en forma cerrada: el rumbo (en múltiplos de 90°)
/// es el número de unos del código Gray de k. Por eso, en paralelo, cada bloque puede
/// calcularse sin conocer los anteriores.
fn direccion(k: usize) -> Vec2 {
    match (k ^ (k >> 1)).count_ones() % 4 {
        0 => egui::vec2(0.0, -PASO_MUNDO),
        1 => egui::vec2(PASO_MUNDO, 0.0),
        2 => egui::vec2(0.0, PASO_MUNDO),
        _ => egui::vec2(-PASO_MUNDO, 0.0),
    }
}

pub struct Dragon {
    escena: Escena,
    paralelo: bool,
    curva: Curva,
}

impl Default for Dragon {
    fn default() -> Self {
        Self {
            escena: Escena::new(12, 200.0),
            paralelo: false,
            curva: Curva::vacia(),
        }
    }
}

impl Dragon {
    /// SECUENCIAL: un único recorrido que gira según la secuencia de plegado de papel.
    fn generar_secuencial(&mut self) {
        let n = 1usize << self.escena.profundidad; // segmentos
        let mut puntos: Vec<Pos2> = Vec::with_capacity(n + 1);

        let mut p = ORIGEN.to_pos2();
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
    }

    /// PARALELO: suma de prefijos (scan) en 3 fases:
    ///  1. (paralela)   cada bloque suma sus desplazamientos
    ///  2. (secuencial) prefijos sobre las pocas sumas de bloque
    ///  3. (paralela)   cada bloque escribe sus puntos partiendo de su prefijo
    fn generar_paralelo(&mut self) {
        let w = hilos::trabajadores();
        let n = 1usize << self.escena.profundidad;
        let tam = ((n + w - 1) / w).max(1);

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
        let mut acc = ORIGEN;
        for s in &sumas {
            inicios.push(acc);
            acc += *s;
        }

        let mut puntos = vec![Pos2::ZERO; n + 1];
        puntos[0] = ORIGEN.to_pos2();
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

        self.curva = Curva::bloques(puntos, w);
    }
}

impl Fractal for Dragon {
    fn nombre(&self) -> &'static str {
        "Curva del Dragón"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        self.escena.controles(ui, 18, &mut self.paralelo, |_, _| {});
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        if self.escena.preparar(ui, rect, self.paralelo) {
            if self.paralelo {
                self.generar_paralelo();
            } else {
                self.generar_secuencial();
            }
        }

        self.curva.mostrar(
            ui,
            rect,
            &mut self.escena,
            PASO_MUNDO,
            1.5,
            Color32::from_rgb(255, 200, 100),
            self.paralelo,
        );
    }
}
