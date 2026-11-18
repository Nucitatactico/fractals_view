use eframe::egui;
use super::Fractal;
use crate::camara::Camara;
use rayon::prelude::*;
use std::thread;

#[derive(Clone, Copy)]
pub struct Segmento {
    a: egui::Pos2,
    b: egui::Pos2,
    color: egui::Color32,
    grosor: f32,
    hilo: u8,
    tiempo_nacimiento: f32,
}

pub struct CopoKochParalelo {
    pub profundidad: u32,
    pub camara: Camara,
    pub animando: bool,
    pub tiempo_inicio: f64,
    pub segmentos_por_segundo: f32,
    pub geometria: Vec<Segmento>,
    pub firma_geometria: Option<u32>,
}

impl Default for CopoKochParalelo {
    fn default() -> Self {
        let mut app = Self {
            profundidad: 5,
            camara: Camara::default(),
            animando: true,
            tiempo_inicio: 0.0,
            segmentos_por_segundo: 200.0,
            geometria: Vec::new(),
            firma_geometria: None,
        };
        app.generar_geometria();
        app
    }
}

impl CopoKochParalelo {
    fn generar_geometria(&mut self) {
        self.geometria.clear();
        let tamano = 200.0_f32;
        let radio = tamano / 3.0_f32.sqrt();

        let v0 = egui::pos2(0.0, -radio);
        let v1 = egui::pos2(radio * 1.5_f32.sqrt(), radio * 0.5);
        let v2 = egui::pos2(-radio * 1.5_f32.sqrt(), radio * 0.5);

        let prof = self.profundidad;

        let estimado = 3usize.saturating_mul(4usize.saturating_pow(prof));
        self.geometria.reserve(estimado);

        // Tres hilos, uno por lado del triángulo.
        let (lado0, lado1, lado2) = thread::scope(|s| {
            let h0 = s.spawn(|| Self::calcular_lado(v0, v1, prof, prof, 0));
            let h1 = s.spawn(|| Self::calcular_lado(v1, v2, prof, prof, 1));
            let h2 = s.spawn(|| Self::calcular_lado(v2, v0, prof, prof, 2));
            (h0.join().unwrap(), h1.join().unwrap(), h2.join().unwrap())
        });

        self.geometria.extend(lado0);
        self.geometria.extend(lado1);
        self.geometria.extend(lado2);

        self.firma_geometria = Some(self.profundidad);
    }

    fn calcular_lado(
        a: egui::Pos2,
        b: egui::Pos2,
        prof_actual: u32,
        prof_max: u32,
        hilo: u8,
    ) -> Vec<Segmento> {
        // Color base según hilo, con variación de tono por profundidad.
        let base_hue = match hilo {
            0 => 0.55, // cian
            1 => 0.05, // naranja
            _ => 0.30, // verde
        };
        let prof_desde_raiz = prof_max - prof_actual;
        let hue = (base_hue + prof_desde_raiz as f32 * 0.02).rem_euclid(1.0);
        let sat = 0.8;
        let val = 0.9 - (prof_desde_raiz as f32 * 0.02).min(0.3);
        let color = egui::Color32::from(egui::ecolor::Hsva::new(hue, sat, val, 1.0));
        let grosor = (2.0 / (prof_desde_raiz as f32 + 1.0)).max(0.5);

        // Caso base: segmento hoja.
        if prof_actual == 0 {
            return vec![Segmento {
                a,
                b,
                color,
                grosor,
                hilo,
                tiempo_nacimiento: 0.0,
            }];
        }

        // Subdividir.
        let v = b - a;
        let p1 = a + v / 3.0;
        let p3 = a + v * (2.0 / 3.0);

        let angulo = std::f32::consts::FRAC_PI_3;
        let (sin, cos) = angulo.sin_cos();
        let v_tercio = v / 3.0;
        let rotado = egui::vec2(
            v_tercio.x * cos - v_tercio.y * sin,
            v_tercio.x * sin + v_tercio.y * cos,
        );
        let p2 = p1 + rotado;

        let subtareas = [(a, p1), (p1, p2), (p2, p3), (p3, b)];

        let hijos: Vec<Vec<Segmento>> = subtareas
            .par_iter()
            .map(|&(sa, sb)| Self::calcular_lado(sa, sb, prof_actual - 1, prof_max, hilo))
            .collect();

        let mut resultado: Vec<Segmento> = Vec::new();
        for sub in hijos {
            resultado.extend(sub);
        }

        let dt = 1.0_f32;
        for (i, seg) in resultado.iter_mut().enumerate() {
            seg.tiempo_nacimiento = i as f32 * dt;
        }

        resultado
    }
}

impl Fractal for CopoKochParalelo {
    fn nombre(&self) -> &'static str {
        "Copo de Koch (Paralelo)"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui
                .add(egui::Slider::new(&mut self.profundidad, 1..=12))
                .changed()
            {
                self.firma_geometria = None;
                self.animando = true;
                self.tiempo_inicio = ui.input(|i| i.time);
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.segmentos_por_segundo, 50.0..=5000.0));
            if ui.button("Repetir animación").clicked() {
                self.animando = true;
                self.tiempo_inicio = ui.input(|i| i.time);
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
            ui.label(format!("Segmentos: {}", self.geometria.len()));
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let centro = rect.center();
        self.camara.procesar_zoom(ui, centro);
        self.camara.procesar_pan(ui);

        if self.firma_geometria != Some(self.profundidad) {
            self.generar_geometria();
        }

        // Reloj global: cuántos segmentos "por hilo" ya deberían estar.
        let reloj = if self.animando {
            let ahora = ui.input(|i| i.time);
            let transcurrido = (ahora - self.tiempo_inicio) as f32;
            let n = transcurrido * self.segmentos_por_segundo;
            let total_por_hilo = self.geometria.len() / 3;
            if n >= total_por_hilo as f32 {
                self.animando = false;
                f32::INFINITY
            } else {
                ui.ctx().request_repaint();
                n
            }
        } else {
            f32::INFINITY
        };

        let painter = ui.painter();
        let escala = self.camara.zoom;
        let ox = centro.x + self.camara.offset.x;
        let oy = centro.y + self.camara.offset.y;

        for seg in &self.geometria {
            // Cada hilo tiene su propio reloj ligeramente desfasado para que
            // se vea que avanzan en paralelo.
            let desfase_hilo = seg.hilo as f32 * 3.0;
            if seg.tiempo_nacimiento + desfase_hilo > reloj {
                continue;
            }

            let p1 = egui::pos2(ox + seg.a.x * escala, oy + seg.a.y * escala);
            let p2 = egui::pos2(ox + seg.b.x * escala, oy + seg.b.y * escala);

            // Pequeño "brillo" extra justo cuando el segmento nace, para
            // enfatizar el trabajo de los hilos.
            let edad = reloj - (seg.tiempo_nacimiento + desfase_hilo);
            let grosor = if edad < 5.0 {
                seg.grosor * 1.5
            } else {
                seg.grosor
            };

            painter.line_segment([p1, p2], egui::Stroke::new(grosor, seg.color));
        }
    }
}
