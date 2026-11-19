use super::arbol_pitagoras::{
    base_raiz, crear_nodo, generar_rec, A45, ANGULO_INICIAL, LADO_MIN, LADO_RAIZ, R,
};
use super::Fractal;
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::dibujo::{self, Cuadrado};
use eframe::egui::{self, Pos2};
use rayon::prelude::*;
use std::time::Instant;

/// Hasta este nivel se bifurca con `rayon::join`; más abajo, recursión secuencial.
const NIVEL_SECUENCIAL: u32 = 10;

pub struct ArbolPitagorasParalelo {
    pub profundidad: u32,
    pub camara: Camara,
    /// `velocidad` = niveles del árbol que aparecen por segundo.
    anim: Animacion,
    /// Ordenada por nivel: los cuadrados del nivel k son `geometria[limites[k]..limites[k+1]]`.
    geometria: Vec<Cuadrado>,
    limites: Vec<usize>,
    firma: Option<u32>,
    ms_generacion: f32,
}

impl Default for ArbolPitagorasParalelo {
    fn default() -> Self {
        Self {
            profundidad: 10,
            camara: Camara::default(),
            anim: Animacion::new(6.0),
            geometria: Vec::new(),
            limites: vec![0],
            firma: None,
            ms_generacion: 0.0,
        }
    }
}

/// Fork-join real: cada subárbol alto se reparte entre los hilos de rayon.
fn generar_par(base: Pos2, lado: f32, angulo: f32, nivel: u32, prof_max: u32) -> Vec<Cuadrado> {
    if nivel >= prof_max || lado < LADO_MIN {
        return Vec::new();
    }
    if nivel >= NIVEL_SECUENCIAL {
        let mut v = Vec::new();
        generar_rec(&mut v, base, lado, angulo, nivel, prof_max);
        return v;
    }

    let nodo = crear_nodo(base, lado, angulo, nivel, prof_max);
    let l = lado * R;
    let (base_izq, base_der) = (nodo.base_izq, nodo.base_der);

    let (izq, der) = rayon::join(
        || generar_par(base_izq, l, angulo + A45, nivel + 1, prof_max),
        || generar_par(base_der, l, angulo - A45, nivel + 1, prof_max),
    );

    let mut v = Vec::with_capacity(1 + izq.len() + der.len());
    v.push(nodo.cuadrado);
    v.extend(izq);
    v.extend(der);
    v
}

impl ArbolPitagorasParalelo {
    fn generar(&mut self) {
        let t0 = Instant::now();

        self.geometria = generar_par(
            base_raiz(self.profundidad),
            LADO_RAIZ,
            ANGULO_INICIAL,
            0,
            self.profundidad,
        );
        // Orden estable por nivel para poder revelar el árbol "por capas".
        self.geometria.par_sort_by_key(|c| c.nivel);

        let niveles = self
            .geometria
            .last()
            .map(|c| c.nivel as usize + 1)
            .unwrap_or(0);
        self.limites = vec![0; niveles + 1];
        for c in &self.geometria {
            self.limites[c.nivel as usize + 1] += 1;
        }
        for i in 1..=niveles {
            self.limites[i] += self.limites[i - 1];
        }

        self.firma = Some(self.profundidad);
        self.ms_generacion = t0.elapsed().as_secs_f32() * 1000.0;
    }
}

impl Fractal for ArbolPitagorasParalelo {
    fn nombre(&self) -> &'static str {
        "Árbol de Pitágoras (Simulación Fork-Join)"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=24)).changed() {
                self.anim.reiniciar();
            }
            ui.label("Niveles/s:");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 1.0..=60.0));
            if ui.button("Repetir animación").clicked() {
                self.anim.reiniciar();
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
            ui.label(format!(
                "Nodos: {} | Generación: {:.1} ms",
                self.geometria.len(),
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

        let niveles = self.limites.len() - 1;
        let k = self.anim.progreso(ui, niveles);
        let hasta = self.limites[k];

        let painter = ui.painter_at(rect);
        dibujo::cuadrados(&painter, &self.camara, centro, rect, &self.geometria[..hasta]);
    }
}
