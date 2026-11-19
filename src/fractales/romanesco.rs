use super::Fractal;
use crate::animacion::Animacion;
use crate::camara::Camara;
use eframe::egui::{self, Color32, Pos2, Shape, Stroke};
use std::time::Instant;

/// Tamaño del romanesco en unidades del mundo (pensado para una ventana de ~1000 px de alto).
const RADIO_BASE: f32 = 370.0;
const BASE_Y: f32 = 264.0;

#[derive(Clone, Copy)]
struct Bulto {
    pos: Pos2,
    radio: f32,
    color: Color32,
}

pub struct Romanesco {
    pub profundidad: u32,
    pub camara: Camara,
    anim: Animacion,
    /// Todos los bultos en coordenadas del mundo, en el orden en que se "dibujan".
    bultos: Vec<Bulto>,
    firma: Option<u32>,
    ms_generacion: f32,
}

impl Default for Romanesco {
    fn default() -> Self {
        Self {
            profundidad: 2,
            camara: Camara::default(),
            anim: Animacion::new(400.0),
            bultos: Vec::new(),
            firma: None,
            ms_generacion: 0.0,
        }
    }
}

impl Romanesco {
    const ANGULO_DORADO: f32 = 2.399_963_2;
    const NIVELES: usize = 8;
    const FIB: [u32; Self::NIVELES] = [5, 8, 13, 21, 21, 13, 8, 5];
    const RADIO_MINIMO_PX: f32 = 1.5;
    /// 1.0 = niveles tangentes, <1.0 = se solapan, >1.0 = separados.
    const SOLAPE_Y: f32 = 0.7;

    fn anillo(nivel: usize, radio_base: f32) -> (f32, f32, u32) {
        let t = nivel as f32 / (Self::NIVELES as f32 - 1.0).max(1.0);
        let radio_nivel = radio_base * (1.0 - t);

        let fib = Self::FIB[nivel.min(Self::FIB.len() - 1)];
        let cap = (radio_nivel * std::f32::consts::PI / Self::RADIO_MINIMO_PX) as u32;
        let num_bultos = fib.min(cap).max(1);

        let radio_bulto = (std::f32::consts::PI * radio_nivel / num_bultos as f32) * 1.05;
        (radio_nivel, radio_bulto, num_bultos)
    }

    fn alturas_niveles(radio_base: f32) -> [f32; Self::NIVELES] {
        let mut ys = [0.0f32; Self::NIVELES];
        let mut cum = 0.0f32;
        for nivel in 0..Self::NIVELES {
            ys[nivel] = cum;
            let (_, radio_bulto, _) = Self::anillo(nivel, radio_base);
            cum += radio_bulto * Self::SOLAPE_Y;
        }
        ys
    }

    /// Antes se hacía en cada frame (contar + dibujar). Ahora se calcula una vez.
    fn generar_torre(out: &mut Vec<Bulto>, centro: Pos2, radio_base: f32, profundidad: u32) {
        if profundidad == 0 || radio_base < Self::RADIO_MINIMO_PX {
            return;
        }
        let ys = Self::alturas_niveles(radio_base);

        for nivel in 0..Self::NIVELES {
            let t = nivel as f32 / (Self::NIVELES as f32 - 1.0).max(1.0);
            let y = centro.y - ys[nivel];
            let (radio_nivel, radio_bulto, num_bultos) = Self::anillo(nivel, radio_base);
            if radio_bulto < Self::RADIO_MINIMO_PX {
                continue;
            }

            let offset_nivel = nivel as f32 * Self::ANGULO_DORADO * 0.5;
            let color = Color32::from_rgb(
                (40.0 + 100.0 * t) as u8,
                (110.0 + 100.0 * t) as u8,
                (30.0 + 60.0 * t) as u8,
            );

            for i in 0..num_bultos {
                let angulo = i as f32 * Self::ANGULO_DORADO + offset_nivel;
                let offset_x = angulo.cos() * radio_nivel;
                let offset_z = angulo.sin() * radio_nivel * 0.35;
                let pos = egui::pos2(centro.x + offset_x, y + offset_z);

                out.push(Bulto { pos, radio: radio_bulto, color });

                if profundidad > 1 && radio_bulto > 3.0 {
                    Self::generar_torre(out, pos, radio_bulto * 0.85, profundidad - 1);
                }
            }
        }
    }

    fn generar(&mut self) {
        let t0 = Instant::now();
        self.bultos.clear();
        Self::generar_torre(
            &mut self.bultos,
            egui::pos2(0.0, BASE_Y),
            RADIO_BASE,
            self.profundidad,
        );
        self.firma = Some(self.profundidad);
        self.ms_generacion = t0.elapsed().as_secs_f32() * 1000.0;
    }
}

impl Fractal for Romanesco {
    fn nombre(&self) -> &'static str {
        "Brócoli Romanesco"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=3)).changed() {
                self.anim.reiniciar();
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 10.0..=20000.0).logarithmic(true));
            if ui.button("Repetir animación").clicked() {
                self.anim.reiniciar();
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
            ui.label(format!(
                "Bultos: {} | Generación: {:.1} ms",
                self.bultos.len(),
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

        let n = self.anim.progreso(ui, self.bultos.len());
        let origen = self.camara.origen(centro);
        let z = self.camara.zoom;

        let borde = Stroke::new(0.6, Color32::from_rgb(20, 60, 20));
        let mut shapes: Vec<Shape> = Vec::with_capacity(n * 2);

        for b in &self.bultos[..n] {
            let r = b.radio * z;
            if r < 0.5 {
                continue; // LOD: invisible
            }
            let pos = origen + b.pos.to_vec2() * z;
            if !rect.expand(r).contains(pos) {
                continue; // culling
            }
            shapes.push(Shape::circle_filled(pos, r, b.color));
            if r > 2.5 {
                shapes.push(Shape::circle_stroke(pos, r, borde));
            }
        }

        ui.painter_at(rect).extend(shapes);
    }
}
