use super::{koch_puntos, triangulo_koch, Fractal, LADO_TRIANGULO_KOCH};
use crate::curva::Curva;
use crate::escena::Escena;
use crate::hilos;
use eframe::egui::{self, Color32, Pos2};
use rayon::prelude::*;

/// Por debajo de este nivel restante se calcula en secuencial
const UMBRAL_PARALELO: u32 = 12;

/// PARALELO: rellena `out` (longitud exacta 4^prof) con el punto de inicio de cada segmento hoja.
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

pub struct CopoKoch {
    escena: Escena,
    paralelo: bool,
    curva: Curva,
}

impl Default for CopoKoch {
    fn default() -> Self {
        Self {
            escena: Escena::new(5, 300.0),
            paralelo: false,
            curva: Curva::vacia(),
        }
    }
}

impl CopoKoch {
    /// SECUENCIAL: recursión clásica que va empujando puntos al Vec.
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

    fn generar_secuencial(&mut self) {
        let prof = self.escena.profundidad;
        let [v0, v1, v2] = triangulo_koch();
        let n = 3 * 4usize.pow(prof);

        let mut puntos = Vec::with_capacity(n + 1);
        Self::generar_lado(v0, v1, prof, &mut puntos);
        Self::generar_lado(v1, v2, prof, &mut puntos);
        Self::generar_lado(v2, v0, prof, &mut puntos);
        puntos.push(v0); // cierra el contorno

        self.curva = Curva::secuencial(puntos);
    }

    fn generar_paralelo(&mut self) {
        let prof = self.escena.profundidad;
        let vertices = triangulo_koch();
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

        // Visualización: el contorno se reparte en bloques, uno por trabajador.
        self.curva = Curva::bloques(puntos, hilos::trabajadores());
    }
}

impl Fractal for CopoKoch {
    fn nombre(&self) -> &'static str {
        "Copo de Koch"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        self.escena.controles(ui, 12, &mut self.paralelo, |_, _| {});
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        if self.escena.preparar(ui, rect, self.paralelo) {
            if self.paralelo {
                self.generar_paralelo();
            } else {
                self.generar_secuencial();
            }
        }

        let prof = self.escena.profundidad;
        let largo_hoja = LADO_TRIANGULO_KOCH / 3f32.powi(prof as i32);
        let grosor = (2.0 / (prof as f32 + 1.0)).max(0.5);
        self.curva.mostrar(
            ui,
            rect,
            &mut self.escena,
            largo_hoja,
            grosor,
            Color32::from_rgb(100, 200, 255),
            self.paralelo,
        );
    }
}
impl CopoKoch {
    pub fn medir(profundidad: u32, paralelo: bool) -> usize {
        use rayon::prelude::*;

        fn subdividir_segmento(a: egui::Pos2, b: egui::Pos2, nivel: u32, out: &mut Vec<(egui::Pos2, egui::Pos2)>) {
            if nivel == 0 {
                out.push((a, b));
                return;
            }
            let (p1, pico, p2) = crate::fractales::koch_puntos(a, b);
            subdividir_segmento(a, p1, nivel - 1, out);
            subdividir_segmento(p1, pico, nivel - 1, out);
            subdividir_segmento(pico, p2, nivel - 1, out);
            subdividir_segmento(p2, b, nivel - 1, out);
        }

        let tri = crate::fractales::triangulo_koch();
        let lados = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];

        let segmentos: Vec<(egui::Pos2, egui::Pos2)> = if paralelo {
            lados
                .par_iter()
                .flat_map(|&(a, b)| {
                    let mut local = Vec::new();
                    subdividir_segmento(a, b, profundidad, &mut local);
                    local
                })
                .collect()
        } else {
            let mut local = Vec::new();
            for (a, b) in lados {
                subdividir_segmento(a, b, profundidad, &mut local);
            }
            local
        };

        segmentos.len()
    }
}
