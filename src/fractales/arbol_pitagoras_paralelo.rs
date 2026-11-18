use eframe::egui;
use super::Fractal;
use crate::camara::Camara;
use std::thread;

#[derive(Clone, Copy)]
pub struct Cuadrado {
    p: [egui::Pos2; 4],
    color: egui::Color32,
    // Tiempo (en unidades arbitrarias) en el que este nodo "nace"
    tiempo_nacimiento: f32,
}

pub struct ArbolPitagorasParalelo {
    pub profundidad: u32,
    pub camara: Camara,
    pub animando: bool,
    pub tiempo_inicio: f64,
    pub nodos_por_segundo: f32,
    pub geometria: Vec<Cuadrado>,
    pub firma_geometria: Option<u32>,
    pub velocidad_simulada: f32, // Qué tan rápido avanza el reloj interno
}

impl Default for ArbolPitagorasParalelo {
    fn default() -> Self {
        let mut app = Self {
            profundidad: 10,
            camara: Camara::default(),
            animando: true,
            tiempo_inicio: 0.0,
            nodos_por_segundo: 200.0,
            geometria: Vec::new(),
            firma_geometria: None,
            velocidad_simulada: 1.0,
        };
        app.generar_geometria();
        app
    }
}

impl ArbolPitagorasParalelo {
    fn generar_geometria(&mut self) {
        self.geometria.clear();
        let lado = 100.0_f32;
        let angulo_inicial = -std::f32::consts::FRAC_PI_2;
        let base = egui::pos2(0.0, 0.0);

        let (sin, cos) = angulo_inicial.sin_cos();
        let v1 = base + egui::vec2(cos * lado, sin * lado);
        let v2 = v1 + egui::vec2(-sin * lado, cos * lado);
        let v3 = base + egui::vec2(-sin * lado, cos * lado);

        self.geometria.push(Cuadrado {
            p: [base, v1, v2, v3],
            color: egui::Color32::from_rgb(200, 200, 200),
            tiempo_nacimiento: 0.0, // La raíz nace en t=0
        });

        if self.profundidad > 1 {
            let prof_restante = self.profundidad - 1;
            let a45 = std::f32::consts::FRAC_PI_4;
            let factor = std::f32::consts::FRAC_1_SQRT_2;
            let punto_medio = v3 + egui::vec2(cos * lado * factor, sin * lado * factor);

            let colores = [
                egui::Color32::from_rgb(50, 150, 255),  // Azul
                egui::Color32::from_rgb(255, 80, 50),   // Rojo
                egui::Color32::from_rgb(50, 255, 100),  // Verde
                egui::Color32::from_rgb(255, 200, 50),  // Amarillo
                egui::Color32::from_rgb(200, 50, 255),  // Morado
                egui::Color32::from_rgb(50, 255, 255),  // Cian
            ];

            let (izq, der) = thread::scope(|s| {
                let h_izq = s.spawn(|| {
                    Self::calcular_rama(
                        v3, lado * a45.cos(), angulo_inicial + a45, 
                        prof_restante, self.profundidad, 
                        colores[0],
                        0.1, // Tiempo base
                        1.0  // Factor de velocidad
                    )
                });
                let h_der = s.spawn(|| {
                    Self::calcular_rama(
                        punto_medio, lado * a45.sin(), angulo_inicial - a45, 
                        prof_restante, self.profundidad, 
                        colores[1],
                        0.1, 
                        1.0
                    )
                });
                (h_izq.join().unwrap(), h_der.join().unwrap())
            });

            self.geometria.extend(izq);
            self.geometria.extend(der);
        }

        self.firma_geometria = Some(self.profundidad);
    }

    fn calcular_rama(
        base: egui::Pos2, 
        lado: f32, 
        angulo: f32, 
        prof: u32, 
        prof_max: u32,
        color_base: egui::Color32,
        tiempo_padre: f32,
        factor_velocidad: f32,
    ) -> Vec<Cuadrado> {
        if prof == 0 || lado < 0.5 {
            return Vec::new();
        }
        let mut out = Vec::new();

        let (sin, cos) = angulo.sin_cos();
        let v1 = base + egui::vec2(cos * lado, sin * lado);
        let v2 = v1 + egui::vec2(-sin * lado, cos * lado);
        let v3 = base + egui::vec2(-sin * lado, cos * lado);

        // Cuanto más profundo, más rápido avanza el reloj (simula hilos trabajando solos)
        // pero el nacimiento depende del tiempo del padre.
        let tiempo_actual = tiempo_padre + 0.05 * factor_velocidad;
        
        // Degradado a blanco
        let t = (prof_max - prof) as f32 / prof_max.max(1) as f32;
        let color = egui::Color32::from_rgb(
            (color_base.r() as f32 + (255.0 - color_base.r() as f32) * t) as u8,
            (color_base.g() as f32 + (255.0 - color_base.g() as f32) * t) as u8,
            (color_base.b() as f32 + (255.0 - color_base.b() as f32) * t) as u8,
        );

        out.push(Cuadrado { 
            p: [base, v1, v2, v3], 
            color,
            tiempo_nacimiento: tiempo_actual 
        });

        if prof > 1 {
            let a45 = std::f32::consts::FRAC_PI_4;
            let factor = std::f32::consts::FRAC_1_SQRT_2;
            let punto_medio = v3 + egui::vec2(cos * lado * factor, sin * lado * factor);

            // Asignamos colores según la profundidad para simular la apertura de hilos
            let color_hijo = if prof > prof_max - 4 {
                let idx = (prof_max - prof) as usize % 6;
                match idx {
                    0 => egui::Color32::from_rgb(50, 255, 100), // Verde
                    1 => egui::Color32::from_rgb(255, 200, 50), // Amarillo
                    _ => color_base,
                }
            } else {
                color_base
            };

            // El factor de velocidad aumenta al bajar de nivel (hilo más independiente)
            let nuevo_factor = factor_velocidad * 1.2;

            let mut izq = Self::calcular_rama(
                v3, lado * a45.cos(), angulo + a45, prof - 1, prof_max, 
                color_hijo, tiempo_actual, nuevo_factor
            );
            let mut der = Self::calcular_rama(
                punto_medio, lado * a45.sin(), angulo - a45, prof - 1, prof_max,
                color_hijo, tiempo_actual, nuevo_factor
            );

            out.append(&mut izq);
            out.append(&mut der);
        }

        out
    }
}

impl Fractal for ArbolPitagorasParalelo {
    fn nombre(&self) -> &'static str { "Árbol de Pitágoras (Simulación Fork-Join)" }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=50)).changed() {
                self.firma_geometria = None;
                self.animando = true;
                let t = ui.input(|i| i.time);
                self.tiempo_inicio = t;
            }
            ui.label("Velocidad visual:");
            ui.add(egui::Slider::new(&mut self.velocidad_simulada, 0.1..=5.0));
            if ui.button("Repetir animación").clicked() {
                self.animando = true;
                let t = ui.input(|i| i.time);
                self.tiempo_inicio = t;
            }
            if ui.button("Reset zoom").clicked() { self.camara.reset(); }
            ui.label(format!("Nodos: {}", self.geometria.len()));
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let centro = rect.center();
        self.camara.procesar_zoom(ui, centro);
        self.camara.procesar_pan(ui);

        if self.firma_geometria != Some(self.profundidad) {
            self.generar_geometria();
        }

        // Calculamos el tiempo simulado transcurrido
        let tiempo_simulado = if self.animando {
            let ahora = ui.input(|i| i.time);
            let transcurrido = (ahora - self.tiempo_inicio) as f32 * self.velocidad_simulada;
            
            // Si el tiempo simulado es muy grande, ya terminó
            if transcurrido > 10.0 { 
                self.animando = false;
                f32::INFINITY 
            } else {
                ui.ctx().request_repaint();
                transcurrido
            }
        } else {
            f32::INFINITY
        };

        let painter = ui.painter();
        let r = std::f32::consts::FRAC_1_SQRT_2;
        let n = self.profundidad as f32;
        let altura_mundo = 100.0 * (1.0 - r.powf(n)) / (1.0 - r);

        let base_x = centro.x - 100.0 * self.camara.zoom + self.camara.offset.x;
        let base_y = centro.y + (altura_mundo / 2.0) * self.camara.zoom + self.camara.offset.y;

        // Dibujamos solo los cuadrados cuyo tiempo de nacimiento ya pasó
        for c in &self.geometria {
            if c.tiempo_nacimiento > tiempo_simulado {
                continue; // Aún no ha nacido
            }

            let pts: Vec<egui::Pos2> = c.p.iter().map(|p| {
                egui::pos2(
                    base_x + p.x * self.camara.zoom,
                    base_y + p.y * self.camara.zoom,
                )
            }).collect();
            
            painter.add(egui::Shape::convex_polygon(
                pts,
                c.color,
                egui::Stroke::new(0.5_f32, egui::Color32::BLACK),
            ));
        }
    }
}
