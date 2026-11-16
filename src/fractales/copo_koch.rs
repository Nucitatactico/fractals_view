use eframe::egui;
use super::Fractal;
use crate::camara::Camara;

pub struct CopoKoch {
    pub profundidad: u32,
    pub camara: Camara,
    pub animando: bool,
    pub tiempo_inicio: f64,
    pub segmentos_por_segundo: f32,
    pub total_segmentos: u32,
}

impl Default for CopoKoch {
    fn default() -> Self {
        Self {
            profundidad: 4,
            camara: Camara::default(),
            animando: true,
            tiempo_inicio: 0.0,
            segmentos_por_segundo: 300.0,
            total_segmentos: 0,
        }
    }
}

impl CopoKoch {
    fn contar_segmentos(profundidad: u32) -> u32 {
        // 3 lados * 4^profundidad. Con profundidad <= 8 no hay overflow.
        3 * 4u32.pow(profundidad)
    }

    fn dibujar_lado(
        painter: &egui::Painter,
        a: egui::Pos2,
        b: egui::Pos2,
        profundidad: u32,
        visitados: &mut u32,
        objetivo: u32,
        color: egui::Color32,
        grosor: f32,
    ) -> bool {
        if profundidad == 0 {
            if *visitados >= objetivo {
                return false;
            }
            *visitados += 1;
            painter.line_segment([a, b], egui::Stroke::new(grosor, color));
            return true;
        }

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

        if !Self::dibujar_lado(painter, a, p1, profundidad - 1, visitados, objetivo, color, grosor) {
            return false;
        }
        if !Self::dibujar_lado(painter, p1, p2, profundidad - 1, visitados, objetivo, color, grosor) {
            return false;
        }
        if !Self::dibujar_lado(painter, p2, p3, profundidad - 1, visitados, objetivo, color, grosor) {
            return false;
        }
        Self::dibujar_lado(painter, p3, b, profundidad - 1, visitados, objetivo, color, grosor)
    }
}

impl Fractal for CopoKoch {
    fn nombre(&self) -> &'static str { "Copo de Koch" }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            ui.add(egui::Slider::new(&mut self.profundidad, 1..=8));
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.segmentos_por_segundo, 50.0..=5000.0));
            if ui.button("Repetir animación").clicked() {
                let t = ui.input(|i| i.time);
                self.animando = true;
                self.tiempo_inicio = t;
                self.total_segmentos = 0;
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let centro = rect.center();
        self.camara.procesar_zoom(ui, centro);
        self.camara.procesar_pan(ui);

        let tamano_efectivo = 200.0 * self.camara.zoom;
        let radio = tamano_efectivo / 3.0_f32.sqrt();

        let v0 = egui::pos2(
            centro.x + self.camara.offset.x,
            centro.y - radio + self.camara.offset.y,
        );
        let v1 = egui::pos2(
            v0.x + radio * 1.5_f32.sqrt(),
            v0.y + radio * 0.5,
        );
        let v2 = egui::pos2(
            v0.x - radio * 1.5_f32.sqrt(),
            v0.y + radio * 0.5,
        );

        if self.total_segmentos == 0 {
            self.total_segmentos = Self::contar_segmentos(self.profundidad);
        }

        let objetivo = if self.animando {
            let ahora = ui.input(|i| i.time);
            let n = ((ahora - self.tiempo_inicio) as f32 * self.segmentos_por_segundo) as u32;
            if n >= self.total_segmentos {
                self.animando = false;
                u32::MAX
            } else {
                ui.ctx().request_repaint();
                n
            }
        } else {
            u32::MAX
        };

        let grosor = (2.0 / (self.profundidad as f32 + 1.0)).max(0.5);
        let color = egui::Color32::from_rgb(100, 200, 255);
        let painter = ui.painter();

        let mut visitados = 0u32;
        if !Self::dibujar_lado(painter, v0, v1, self.profundidad, &mut visitados, objetivo, color, grosor) {
            return;
        }
        if !Self::dibujar_lado(painter, v1, v2, self.profundidad, &mut visitados, objetivo, color, grosor) {
            return;
        }
        Self::dibujar_lado(painter, v2, v0, self.profundidad, &mut visitados, objetivo, color, grosor);
    }
}
