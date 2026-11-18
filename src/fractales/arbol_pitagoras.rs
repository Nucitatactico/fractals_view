use eframe::egui;
use super::Fractal;
use crate::camara::Camara;

pub struct ArbolPitagoras {
    pub profundidad: u32,
    pub camara: Camara,
    pub animando: bool,
    pub tiempo_inicio: f64,
    pub nodos_por_segundo: f32,
    pub total_nodos: u32,
}

impl Default for ArbolPitagoras {
    fn default() -> Self {
        Self {
            profundidad: 10,
            camara: Camara::default(),
            animando: true,
            tiempo_inicio: 0.0,
            nodos_por_segundo: 200.0,
            total_nodos: 0,
        }
    }
}

impl ArbolPitagoras {
    fn contar_nodos(lado: f32, profundidad: u32) -> u32 {
        if profundidad == 0 || lado < 1.0 {
            return 0;
        }
        let a45 = std::f32::consts::FRAC_PI_4;
        1
            + Self::contar_nodos(lado * a45.cos(), profundidad - 1)
            + Self::contar_nodos(lado * a45.sin(), profundidad - 1)
    }

    fn dibujar_rec(
        painter: &egui::Painter,
        base: egui::Pos2,
        lado: f32,
        angulo: f32,
        profundidad: u32,
        profundidad_max: u32,
        visitados: &mut u32,
        objetivo: u32,
    ) -> bool {
        if profundidad == 0 || lado < 1.0 {
            return true;
        }
        if *visitados >= objetivo {
            return false;
        }
        *visitados += 1;

        let (sin, cos) = angulo.sin_cos();
        let v1 = base + egui::vec2(cos * lado, sin * lado);
        let v2 = v1 + egui::vec2(-sin * lado, cos * lado);
        let v3 = base + egui::vec2(-sin * lado, cos * lado);

        // t = 0 en la raíz, t = 1 en las hojas
        let t = if profundidad_max > 1 {
            let d = (profundidad_max - profundidad) as f32;
            let dmax = (profundidad_max - 1) as f32;
            (d / dmax).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Café en la raíz → verde claro en las hojas
        // Café: (139, 90, 43)   Verde: (100, 200, 80)
        let color = egui::Color32::from_rgb(
            (139.0 + (100.0 - 139.0) * t) as u8,
            (90.0 + (200.0 - 90.0) * t) as u8,
            (43.0 + (80.0 - 43.0) * t) as u8,
        );

        painter.add(egui::Shape::convex_polygon(
            vec![base, v1, v2, v3],
            color,
            egui::Stroke::new(0.5_f32, egui::Color32::BLACK),
        ));

        let a45 = std::f32::consts::FRAC_PI_4;
        let factor = std::f32::consts::FRAC_1_SQRT_2;

        let seguir = Self::dibujar_rec(
            painter, v3, lado * a45.cos(), angulo + a45,
            profundidad - 1, profundidad_max, visitados, objetivo,
        );
        if !seguir { return false; }

        let punto_medio = v3 + egui::vec2(cos * lado * factor, sin * lado * factor);
        Self::dibujar_rec(
            painter, punto_medio, lado * a45.sin(), angulo - a45,
            profundidad - 1, profundidad_max, visitados, objetivo,
        )
    }
}

impl Fractal for ArbolPitagoras {
    fn nombre(&self) -> &'static str { "Árbol de Pitágoras" }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            ui.add(egui::Slider::new(&mut self.profundidad, 1..=30));
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.nodos_por_segundo, 10.0..=2000.0));
            if ui.button("Repetir animación").clicked() {
                let t = ui.input(|i| i.time);
                self.animando = true;
                self.tiempo_inicio = t;
                self.total_nodos = 0;
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

        let lado_base = 100.0_f32;
        let r = std::f32::consts::FRAC_1_SQRT_2;
        let n = self.profundidad as f32;
        let lado_efectivo = lado_base * self.camara.zoom;
        let altura = lado_efectivo * (1.0 - r.powf(n)) / (1.0 - r);

        let base = egui::pos2(
            centro.x - lado_efectivo + self.camara.offset.x,
            centro.y + altura / 2.0 + self.camara.offset.y,
        );
        let angulo_inicial = -std::f32::consts::FRAC_PI_2;

        let objetivo = if self.animando {
            let ahora = ui.input(|i| i.time);
            let n = ((ahora - self.tiempo_inicio) as f32 * self.nodos_por_segundo) as u32;
            if self.total_nodos == 0 {
                self.total_nodos = Self::contar_nodos(lado_efectivo, self.profundidad);
            }
            if n >= self.total_nodos {
                self.animando = false;
                u32::MAX
            } else {
                ui.ctx().request_repaint();
                n
            }
        } else {
            u32::MAX
        };

        let mut visitados = 0u32;
        Self::dibujar_rec(
            ui.painter(), base, lado_efectivo, angulo_inicial,
            self.profundidad, self.profundidad, &mut visitados, objetivo,
        );
    }
}
