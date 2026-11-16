use eframe::egui;
use super::Fractal;
use crate::camara::Camara;

pub struct Romanesco {
    pub profundidad: u32,
    pub camara: Camara,
    pub animando: bool,
    pub tiempo_inicio: f64,
    pub bultos_por_segundo: f32,
}

impl Default for Romanesco {
    fn default() -> Self {
        Self {
            profundidad: 2,
            camara: Camara::default(),
            animando: true,
            tiempo_inicio: 0.0,
            bultos_por_segundo: 40.0,
        }
    }
}

impl Romanesco {
    const ANGULO_DORADO: f32 = 2.399_963_2;
    /// Factor de encogimiento por nivel. Altura y radio lo usan por igual,
    /// así la torre es un cono uniforme.
    const K: f32 = 0.85;

    /// Cuenta los bultos que se dibujarían con esta configuración.
    /// Replica EXACTAMENTE la lógica de `dibujar_torre`.
    fn contar_bultos(_altura: f32, radio_base: f32, profundidad: u32) -> u32 {
        if profundidad == 0 || radio_base < 1.0 {
            return 0;
        }
        let niveles = 3 + profundidad;
        let mut total = 0u32;
        for nivel in 0..niveles {
            let factor = Self::K.powi(nivel as i32);
            let radio_nivel = radio_base * factor;
            let num_bultos = (radio_nivel / radio_base * 8.0).max(2.0) as i32;
            for _ in 0..num_bultos {
                total += 1;
                let radio_bulto = radio_nivel * 0.45;
                if profundidad > 1 && radio_bulto > 2.0 {
                    total += Self::contar_bultos(
                        _altura * 0.25,
                        radio_bulto * 0.8,
                        profundidad - 1,
                    );
                }
            }
        }
        total
    }

    fn dibujar_torre(
        painter: &egui::Painter,
        centro: egui::Pos2,
        altura: f32,
        radio_base: f32,
        profundidad: u32,
        visitados: &mut u32,
        objetivo: u32,
    ) -> bool {
        if profundidad == 0 || radio_base < 1.0 {
            return true;
        }

        let niveles = 3 + profundidad;
        let norm = 1.0 - Self::K.powi(niveles as i32);

        for nivel in 0..niveles {
            let factor = Self::K.powi(nivel as i32);
            // t = 0 en la base, 1 en la punta
            let t = (1.0 - factor) / norm;
            let y = centro.y - altura * t;
            let radio_nivel = radio_base * factor;

            // Bultos proporcionales al radio: anillos pequeños → menos bultos
            let num_bultos = (radio_nivel / radio_base * 8.0).max(2.0) as i32;

            for i in 0..num_bultos {
                if *visitados >= objetivo {
                    return false;
                }
                *visitados += 1;

                let angulo = i as f32 * Self::ANGULO_DORADO + nivel as f32 * 0.5;
                let radio_bulto = radio_nivel * 0.45;

                let offset_x = angulo.cos() * radio_nivel * 0.7;
                let offset_z = angulo.sin() * radio_nivel * 0.7;

                let pos = egui::pos2(centro.x + offset_x, y + offset_z * 0.3);

                // Verde oscuro en la base → verde claro en la punta
                let color = egui::Color32::from_rgb(
                    (60.0 + 80.0 * (1.0 - t)) as u8,
                    (140.0 + 80.0 * (1.0 - t)) as u8,
                    (50.0 + 40.0 * (1.0 - t)) as u8,
                );

                painter.circle_filled(pos, radio_bulto, color);
                painter.circle_stroke(
                    pos,
                    radio_bulto,
                    egui::Stroke::new(0.5, egui::Color32::from_rgb(30, 80, 30)),
                );

                if profundidad > 1 && radio_bulto > 2.0 {
                    let exito = Self::dibujar_torre(
                        painter,
                        egui::pos2(pos.x, pos.y + radio_bulto * 0.5),
                        altura * 0.25,
                        radio_bulto * 0.8,
                        profundidad - 1,
                        visitados,
                        objetivo,
                    );
                    if !exito {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl Fractal for Romanesco {
    fn nombre(&self) -> &'static str { "Brócoli Romanesco" }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=3)).changed() {
                // Reiniciar animación al cambiar profundidad
                let t = ui.input(|i| i.time);
                self.animando = true;
                self.tiempo_inicio = t;
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.bultos_por_segundo, 10.0..=200.0));
            if ui.button("Repetir animación").clicked() {
                let t = ui.input(|i| i.time);
                self.animando = true;
                self.tiempo_inicio = t;
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let centro = rect.center();

        // Pan/zoom siempre activos, como en los otros fractales
        self.camara.procesar_zoom(ui, centro);
        self.camara.procesar_pan(ui);

        let altura_base = rect.height() * 0.6 * self.camara.zoom;
        let radio_base = altura_base * 0.35;

        let base = egui::pos2(
            centro.x + self.camara.offset.x,
            centro.y + altura_base * 0.5 + self.camara.offset.y,
        );

        // Calcular total de bultos cada frame (es barato, solo cuenta).
        // Así, si el usuario hace zoom durante la animación, el objetivo
        // se ajusta y la animación no termina antes de tiempo.
        let total_bultos =
            Self::contar_bultos(altura_base, radio_base, self.profundidad).max(1);

        let objetivo = if self.animando {
            let ahora = ui.input(|i| i.time);
            let n = ((ahora - self.tiempo_inicio) as f32 * self.bultos_por_segundo) as u32;
            if n >= total_bultos {
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
        Self::dibujar_torre(
            ui.painter(),
            base,
            altura_base,
            radio_base,
            self.profundidad,
            &mut visitados,
            objetivo,
        );
    }
}
