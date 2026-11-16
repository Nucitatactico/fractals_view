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
    /// Genera la secuencia de giros para la curva del dragón de orden `n`.
    /// El algoritmo es el mismo que plegar papel: empezamos con un giro derecha (1),
    /// y en cada iteración duplicamos la secuencia insertando alternadamente
    /// giros derecha e izquierda en el medio.
    fn secuencia_giros(n: u32) -> Vec<i8> {
        let mut seq = vec![1i8]; // 1 = derecha, -1 = izquierda
        for _ in 1..n {
            let mut nueva = Vec::with_capacity(seq.len() * 2 + 1);
            nueva.push(1); // giro derecha al inicio
            for &g in &seq {
                nueva.push(g);
                nueva.push(-1); // alternamos
            }
            // La lógica exacta del plegado: 
            // Orden 1: [1]
            // Orden 2: [1, 1, -1]  (derecha, derecha, izquierda)
            // Orden 3: [1, 1, -1, 1, 1, -1, -1] 
            // En realidad, la secuencia se construye insertando alternadamente.
            // Una forma más sencilla: usar la regla de reescritura de L-System.
            // Para evitar errores, usamos la generación directa por plegado:
            // nueva = seq + [1] + reverse(invert(seq))
            let mut inv: Vec<i8> = seq.iter().rev().map(|&x| -x).collect();
            let mut resultado = seq.clone();
            resultado.push(1);
            resultado.append(&mut inv);
            seq = resultado;
        }
        seq
    }

    /// Recorre la secuencia de giros y dibuja los segmentos hasta `objetivo`.
    /// Devuelve true si hay que seguir.
    fn dibujar_hasta(
        painter: &egui::Painter,
        inicio: egui::Pos2,
        paso: f32,
        secuencia: &[i8],
        objetivo: usize,
    ) -> bool {
        let mut pos = inicio;
        let mut angulo = 0.0f32; // radianes, 0 = derecha
        
        // Dirección inicial: hacia arriba (para que la curva se centre mejor)
        // Pero para la curva del dragón estándar, empezamos hacia la derecha.
        // Ajustamos la orientación inicial según el diseño.
        // Para que quede centrada, podemos empezar con ángulo -PI/2 (arriba).
        angulo = -std::f32::consts::FRAC_PI_2; // arriba

        let mut dibujados = 0;
        // El primer segmento (sin giro previo)
        if objetivo > 0 {
            let dir = egui::vec2(angulo.cos(), angulo.sin());
            let fin = pos + dir * paso;
            painter.line_segment(
                [pos, fin],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 200, 100)),
            );
            pos = fin;
            dibujados += 1;
        } else {
            return false;
        }

        for &giro in secuencia {
            if dibujados >= objetivo { return false; }
            // Aplicar giro: +1 = 90° derecha (sentido horario en pantalla, +Y abajo)
            // -1 = 90° izquierda
            // En pantalla, +Y es abajo, así que un giro "derecha" visual es +90° en radianes.
            // Pero la convención matemática es distinta. Ajustamos:
            // Si avanzamos hacia arriba (-PI/2), un giro derecha nos lleva a la derecha (0).
            // Eso es +PI/2 en radianes (de -PI/2 a 0).
            let delta = if giro > 0 { std::f32::consts::FRAC_PI_2 } else { -std::f32::consts::FRAC_PI_2 };
            angulo += delta;

            let dir = egui::vec2(angulo.cos(), angulo.sin());
            let fin = pos + dir * paso;
            painter.line_segment(
                [pos, fin],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 200, 100)),
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

        // La secuencia de giros se calcula cada frame (es barato para n<=16, ~65536 giros)
        let secuencia = Self::secuencia_giros(self.profundidad);
        let total = secuencia.len() + 1; // +1 por el primer segmento

        if self.total_segmentos != total as u32 {
            self.total_segmentos = total as u32;
            // No reiniciamos animación automáticamente aquí para no interrumpir
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

        // El paso (longitud de cada segmento) se ajusta para que la curva quepa.
        // El dragón de orden n tiene un ancho aproximado de sqrt(2^n) * paso.
        // Para que quepa en pantalla, usamos un paso pequeño y dejamos que el zoom haga el resto.
        let paso_base = 4.0;
        let paso = paso_base * self.camara.zoom;

        // Punto de inicio: centro de la pantalla + offset
        // La curva del dragón tiende a crecer hacia la derecha y abajo.
        // Para centrarla mejor, empezamos un poco a la izquierda y arriba.
        let inicio = egui::pos2(
            centro.x + self.camara.offset.x - 100.0 * self.camara.zoom,
            centro.y + self.camara.offset.y + 50.0 * self.camara.zoom,
        );

        let painter = ui.painter();
        Self::dibujar_hasta(painter, inicio, paso, &secuencia, objetivo);
    }
}
