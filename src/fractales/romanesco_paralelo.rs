use super::romanesco::{dibujar_bultos, expandir, hijos, Bulto, BASE_Y, RADIO_BASE};
use super::Fractal;
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::hilos::{self, Reparto};
use eframe::egui;
use rayon::prelude::*;

pub struct RomanescoParalelo {
    pub profundidad: u32,
    pub camara: Camara,
    anim: Animacion,
    bultos: Vec<Bulto>,
    reparto: Reparto,
    trabajadores: usize,
    por_hilo: bool,
    firma: Option<u32>,
}

impl Default for RomanescoParalelo {
    fn default() -> Self {
        Self {
            profundidad: 2,
            camara: Camara::default(),
            anim: Animacion::new(400.0),
            bultos: Vec::new(),
            reparto: Reparto::default(),
            trabajadores: hilos::trabajadores(),
            por_hilo: true,
            firma: None,
        }
    }
}

impl RomanescoParalelo {
    /// Cada "cabeza" del primer anillo es una tarea independiente (ella y todo su
    /// subárbol de cabezas más pequeñas). Se calculan en paralelo con rayon y luego se
    /// reparten entre los trabajadores con una cola dinámica: cada tarea va al hilo
    /// menos cargado, así que las cabezas grandes y las pequeñas se compensan.
    fn generar(&mut self) {
        let w = hilos::trabajadores();
        let prof = self.profundidad;

        let raices = hijos(egui::pos2(0.0, BASE_Y), RADIO_BASE);
        let subs: Vec<Vec<Bulto>> = raices
            .into_par_iter()
            .map(|b| {
                let mut v = Vec::new();
                expandir(&mut v, b, prof);
                v
            })
            .collect();

        let costes: Vec<usize> = subs.iter().map(|s| s.len()).collect();
        let plan = hilos::planificar(&costes, w);

        let total: usize = costes.iter().sum();
        let mut todos = Vec::with_capacity(total);
        let mut reparto = Reparto::nuevo(w);
        for (mut sub, &(hilo, inicio)) in subs.into_iter().zip(plan.iter()) {
            for (j, b) in sub.iter_mut().enumerate() {
                b.hilo = hilo as u8;
                b.tiempo = (inicio + j + 1) as f32;
                reparto.registrar(hilo, b.tiempo);
            }
            todos.extend(sub);
        }

        self.trabajadores = w;
        self.bultos = todos;
        self.reparto = reparto;
        self.firma = Some(prof);
    }
}

impl Fractal for RomanescoParalelo {
    fn nombre(&self) -> &'static str {
        "Brócoli Romanesco (Paralelo)"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=3)).changed() {
                self.anim.reiniciar();
            }
            ui.label("Velocidad (por hilo):");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 10.0..=200000.0).logarithmic(true));
            ui.checkbox(&mut self.por_hilo, "Colorear por hilo");
            if ui.button("Repetir animación").clicked() {
                self.anim.reiniciar();
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        self.camara.interactuar(ui, rect);

        if self.firma != Some(self.profundidad) {
            self.generar();
        }

        let titulo = format!("Paralelo: {} hilos", self.trabajadores);
        dibujar_bultos(
            ui,
            rect,
            &self.camara,
            &mut self.anim,
            &self.bultos,
            &self.reparto,
            self.por_hilo,
            &titulo,
        );
    }
}
