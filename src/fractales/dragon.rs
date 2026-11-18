use eframe::egui;
use super::Fractal;
use crate::camara::Camara;

pub struct Dragon {
    pub profundidad: u32,
    pub camara: Camara,
    pub animando: bool,
    pub tiempo_inicio: f64,
    pub segmentos_por_segundo: f32,
    pub total_segmentos: u32,
}

impl Default for Dragon {
    fn default() -> Self {
        Self {
            profundidad: 12,
            camara: Camara::default(),
            animando: true,
            tiempo_inicio: 0.0,
            segmentos_por_segundo: 200.0,
            total_segmentos: 0,
        }
    }
}

impl Dragon {
    fn secuencia_giros(n: u32) -> Vec<i8> {
        let mut seq = vec![1i8];
        for _ in 1..n {
            let mut inv: Vec<i8> = seq.iter().rev().map(|&x| -x).collect();
            let mut resultado = seq.clone();
            resultado.push(1);
            resultado.append(&mut inv);
            seq = resultado;
        }
        seq
    }

    fn dibujar_hasta(
        painter: &egui::Painter,
        inicio: egui::Pos2,
        paso: f32,
        secuencia: &[i8],
        objetivo: usize,
    ) -> bool {
        let mut pos = inicio;
        let mut angulo = -std::f32::consts::FRAC_PI_2;

        let mut dibujados = 0;
        if objetivo > 0 {
            let dir = egui::vec2(angulo.cos(), angulo.sin());
            let fin = pos + dir * paso;
            painter.line_segment(
                [pos, fin],
                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 200, 100)),
            );
            pos = fin;
            dibujados += 1;
        } else {
            return false;
        }

        for &giro in secuencia {
            if dibujados >= objetivo { return false; }
            let delta = if giro > 0 {
                std::f32::consts::FRAC_PI_2
            } else {
                -std::f32::consts::FRAC_PI_2
            };
            angulo += delta;

            let dir = egui::vec2(angulo.cos(), angulo.sin());
            let fin = pos + dir * paso;
            painter.line_segment(
                [pos, fin],
                egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(255, 200, 100)),
            );
            pos = fin;
            dibujados += 1;
        }
        true
    }
}

impl Fractal for Dragon {
    fn nombre(&self) -> &'static str { "Curva del Dragón" }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=16)).changed() {
                self.total_segmentos = 0;
                self.animando = true;
                let t = ui.input(|i| i.time);
                self.tiempo_inicio = t;
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.segmentos_por_segundo, 50.0..=2000.0));
            if ui.button("Repetir animación").clicked() {
                self.animando = true;
                let t = ui.input(|i| i.time);
                self.tiempo_inicio = t;
            }
            if ui.button("Reset zoom").clicked() { self.camara.reset(); }
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let centro = rect.center();
        self.camara.procesar_zoom(ui, centro);
        self.camara.procesar_pan(ui);

        let secuencia = Self::secuencia_giros(self.profundidad);
        let total = secuencia.len() + 1;

        if self.total_segmentos != total as u32 {
            self.total_segmentos = total as u32;
        }

        let objetivo = if self.animando {
            let ahora = ui.input(|i| i.time);
            let n = ((ahora - self.tiempo_inicio) as f32 * self.segmentos_por_segundo) as usize;
            if n >= total {
                self.animando = false;
                usize::MAX
            } else {
                ui.ctx().request_repaint();
                n
            }
        } else {
            usize::MAX
        };

        let paso = 4.0_f32 * self.camara.zoom;
        let inicio = egui::pos2(
            centro.x + self.camara.offset.x - 100.0 * self.camara.zoom,
            centro.y + self.camara.offset.y + 50.0 * self.camara.zoom,
        );

        let painter = ui.painter();
        Self::dibujar_hasta(painter, inicio, paso, &secuencia, objetivo);
    }
}
