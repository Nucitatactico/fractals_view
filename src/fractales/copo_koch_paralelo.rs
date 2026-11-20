use super::{koch_puntos, triangulo_koch, Fractal, LADO_TRIANGULO_KOCH};
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::curva::Curva;
use crate::hilos;
use eframe::egui::{self, Pos2};
use rayon::prelude::*;

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

pub struct CopoKochParalelo {
    pub profundidad: u32,
    /// Valor del slider; se aplica al soltarlo (así arrastrar no regenera todo).
    pedida: u32,
    pub camara: Camara,
    anim: Animacion,
    curva: Curva,
    trabajadores: usize,
    firma: Option<u32>,
}

impl Default for CopoKochParalelo {
    fn default() -> Self {
        Self {
            profundidad: 5,
            pedida: 5,
            camara: Camara::default(),
            anim: Animacion::new(300.0),
            curva: Curva::vacia(),
            trabajadores: hilos::trabajadores(),
            firma: None,
        }
    }
}

impl CopoKochParalelo {
    fn generar(&mut self) {
        let vertices = triangulo_koch();
        let prof = self.profundidad;
        let n = 4usize.pow(prof);

        // Un buffer único: [lado0 | lado1 | lado2 | punto final]
        let mut puntos = vec![Pos2::ZERO; 3 * n + 1];
        puntos[3 * n] = vertices[0];
        puntos[..3 * n]
            .par_chunks_mut(n)
            .enumerate()
            .for_each(|(i, trozo)| {
                rellenar(vertices[i], vertices[(i + 1) % 3], prof, trozo);
            });

        let w = hilos::trabajadores();
        self.trabajadores = w;
        // Visualización: el contorno se reparte en `w` bloques, uno por trabajador.
        self.curva = Curva::bloques(puntos, w);
        self.firma = Some(prof);
    }
}

impl Fractal for CopoKochParalelo {
    fn nombre(&self) -> &'static str {
        "Copo de Koch (Paralelo)"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            let r = ui.add(egui::Slider::new(&mut self.pedida, 1..=12));
            if r.drag_stopped() || (r.changed() && !r.dragged()) {
                self.profundidad = self.pedida;
                self.anim.reiniciar();
            }
            ui.label("Velocidad (por hilo):");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 50.0..=5000000.0).logarithmic(true));
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

        let largo_px =
            (LADO_TRIANGULO_KOCH / 3f32.powi(self.profundidad as i32) * self.camara.zoom).max(1e-6);
        let paso = ((1.0 / largo_px).ceil() as usize).clamp(1, total.max(1));
        let grosor = (2.0 / (self.profundidad as f32 + 1.0)).max(0.5);

        let painter = ui.painter_at(rect);
        self.curva.dibujar(
            &painter,
            &self.camara,
            centro,
            n,
            self.anim.animando,
            paso,
            grosor,
            hilos::color_hilo,
        );
        let titulo = format!("Paralelo: {} hilos", self.trabajadores);
        hilos::panel(&painter, rect, &titulo, &self.curva.filas(n));
    }
}
