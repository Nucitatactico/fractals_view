use super::Fractal;
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::dibujo::{self, Cuadrado};
use eframe::egui::{self, Color32, Pos2};
use std::f32::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4};
use std::time::Instant;

/// Lado del cuadrado raíz (unidades del mundo).
pub const LADO_RAIZ: f32 = 100.0;
/// No se generan cuadrados más pequeños que esto (en unidades del mundo).
pub const LADO_MIN: f32 = 0.25;
pub const A45: f32 = FRAC_PI_4;
/// cos(45°) = sin(45°): ambos hijos miden lado * R.
pub const R: f32 = FRAC_1_SQRT_2;

pub struct Nodo {
    pub cuadrado: Cuadrado,
    pub base_izq: Pos2,
    pub base_der: Pos2,
}

/// Calcula el cuadrado de un nodo y las bases de sus dos hijos.
pub fn crear_nodo(base: Pos2, lado: f32, angulo: f32, nivel: u32, prof_max: u32) -> Nodo {
    let (sin, cos) = angulo.sin_cos();
    let v1 = base + egui::vec2(cos * lado, sin * lado);
    let v2 = v1 + egui::vec2(-sin * lado, cos * lado);
    let v3 = base + egui::vec2(-sin * lado, cos * lado);
    let medio = v3 + egui::vec2(cos * lado * R, sin * lado * R);

    // t = 0 en la raíz, t = 1 en las hojas. Café -> verde claro.
    let t = if prof_max > 1 {
        (nivel as f32 / (prof_max - 1) as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let color = Color32::from_rgb(
        (139.0 + (100.0 - 139.0) * t) as u8,
        (90.0 + (200.0 - 90.0) * t) as u8,
        (43.0 + (80.0 - 43.0) * t) as u8,
    );

    Nodo {
        cuadrado: Cuadrado {
            p: [base, v1, v2, v3],
            lado,
            color,
            nivel: nivel.min(255) as u8,
        },
        base_izq: v3,
        base_der: medio,
    }
}

/// Recursión secuencial en preorden (DFS). Escribe en coordenadas del mundo.
pub fn generar_rec(
    out: &mut Vec<Cuadrado>,
    base: Pos2,
    lado: f32,
    angulo: f32,
    nivel: u32,
    prof_max: u32,
) {
    if nivel >= prof_max || lado < LADO_MIN {
        return;
    }
    let n = crear_nodo(base, lado, angulo, nivel, prof_max);
    out.push(n.cuadrado);
    let l = lado * R;
    generar_rec(out, n.base_izq, l, angulo + A45, nivel + 1, prof_max);
    generar_rec(out, n.base_der, l, angulo - A45, nivel + 1, prof_max);
}

/// Base del cuadrado raíz para que el árbol quede centrado verticalmente.
pub fn base_raiz(profundidad: u32) -> Pos2 {
    let altura = LADO_RAIZ * (1.0 - R.powi(profundidad as i32)) / (1.0 - R);
    egui::pos2(-LADO_RAIZ, altura / 2.0)
}

pub const ANGULO_INICIAL: f32 = -FRAC_PI_2;

pub struct ArbolPitagoras {
    pub profundidad: u32,
    pub camara: Camara,
    anim: Animacion,
    geometria: Vec<Cuadrado>,
    firma: Option<u32>,
    ms_generacion: f32,
}

impl Default for ArbolPitagoras {
    fn default() -> Self {
        Self {
            profundidad: 10,
            camara: Camara::default(),
            anim: Animacion::new(200.0),
            geometria: Vec::new(),
            firma: None,
            ms_generacion: 0.0,
        }
    }
}

impl ArbolPitagoras {
    fn generar(&mut self) {
        let t0 = Instant::now();
        self.geometria.clear();
        generar_rec(
            &mut self.geometria,
            base_raiz(self.profundidad),
            LADO_RAIZ,
            ANGULO_INICIAL,
            0,
            self.profundidad,
        );
        self.firma = Some(self.profundidad);
        self.ms_generacion = t0.elapsed().as_secs_f32() * 1000.0;
    }
}

impl Fractal for ArbolPitagoras {
    fn nombre(&self) -> &'static str {
        "Árbol de Pitágoras"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=24)).changed() {
                self.anim.reiniciar();
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 10.0..=50000.0).logarithmic(true));
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

        // Animación: se revelan los nodos en el orden en que los visita el DFS.
        let n = self.anim.progreso(ui, self.geometria.len());

        let painter = ui.painter_at(rect);
        dibujo::cuadrados(&painter, &self.camara, centro, rect, &self.geometria[..n]);
    }
}
