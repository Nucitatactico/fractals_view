use eframe::egui;
use super::Fractal;
use crate::camara::Camara;

#[derive(Clone, Copy)]
struct Circulo {
    x: f32,
    y: f32,
    r: f32,
    curvatura: f32, // 1/r, negativa si es el círculo que encierra
}

impl Circulo {
    fn nuevo(x: f32, y: f32, r: f32) -> Self {
        Self { x, y, r, curvatura: 1.0 / r }
    }
    fn nuevo_encerrando(x: f32, y: f32, r: f32) -> Self {
        Self { x, y, r, curvatura: -1.0 / r }
    }
}

pub struct Apolonio {
    pub profundidad: u32,
    pub camara: Camara,
    pub animando: bool,
    pub tiempo_inicio: f64,
    pub circulos_por_segundo: f32,
    pub circulos_generados: Vec<Circulo>, // Cache de la geometría
    pub firma_geometria: Option<(u32, f32)>, // (profundidad, zoom) para invalidar
}

impl Default for Apolonio {
    fn default() -> Self {
        Self {
            profundidad: 5,
            camara: Camara::default(),
            animando: true,
            tiempo_inicio: 0.0,
            circulos_por_segundo: 30.0,
            circulos_generados: Vec::new(),
            firma_geometria: None,
        }
    }
}

impl Apolonio {
    /// Calcula el cuarto círculo tangente a c1, c2, c3 usando el Teorema de Descartes.
    /// Devuelve None si la curvatura es degenerada (línea recta).
    fn cuarto_circulo(c1: &Circulo, c2: &Circulo, c3: &Circulo) -> Option<Circulo> {
        let k1 = c1.curvatura;
        let k2 = c2.curvatura;
        let k3 = c3.curvatura;

        let suma_k = k1 + k2 + k3;
        let prod_k = k1 * k2 + k2 * k3 + k3 * k1;
        let raiz = 2.0 * prod_k.max(0.0).sqrt();

        // Dos soluciones: una es el círculo que ya conocemos, la otra es el nuevo.
        // La que nos interesa suele ser la de mayor curvatura (círculo más pequeño).
        let k4_soluciones = [suma_k + raiz, suma_k - raiz];

        // Coordenadas complejas para el centro
        let z1 = egui::vec2(c1.x, c1.y);
        let z2 = egui::vec2(c2.x, c2.y);
        let z3 = egui::vec2(c3.x, c3.y);

        let mut mejor: Option<Circulo> = None;

        for &k4 in &k4_soluciones {
            if k4.abs() < 1e-4 || k4 <= 0.0 {
                continue; // ignoramos curvaturas negativas o nulas (círculo exterior)
            }
            let r4 = 1.0 / k4;

            // Fórmula del centro usando números complejos (aproximada con vectores 2D)
            // Para simplificar, usamos la propiedad de que el nuevo círculo es tangente
            // y resolvemos numéricamente. Una aproximación es usar la fórmula compleja:
            // z4 = (k1*z1 + k2*z2 + k3*z3 ± 2*sqrt(k1*k2*z1*z2 + ...)) / k4
            // Pero como no tenemos num_complex, usamos una aproximación geométrica:
            // el centro está a distancia r4 de los tres centros.
            // Resolvemos por trilateración.
            let d12 = (z2 - z1).length();
            let d23 = (z3 - z2).length();
            let d31 = (z1 - z3).length();

            // Comprobación rápida: el círculo debe ser tangente a los tres
            // (distancia entre centros = suma o diferencia de radios)
            // Buscamos un punto que equidiste (r_i + r4) de los tres centros.
            // Usamos el método de intersección de circunferencias.
            // Circunferencia 1: centro z1, radio r1 + r4
            // Circunferencia 2: centro z2, radio r2 + r4
            let r1p = c1.r + r4;
            let r2p = c2.r + r4;

            if d12 > r1p + r2p || d12 < (r1p - r2p).abs() {
                continue; // no se intersectan
            }

            // Intersección de dos circunferencias
            let a = (r1p * r1p - r2p * r2p + d12 * d12) / (2.0 * d12);
            let h_sq = r1p * r1p - a * a;
            if h_sq < 0.0 { continue; }
            let h = h_sq.sqrt();

            let dir = (z2 - z1) / d12;
            let perp = egui::vec2(-dir.y, dir.x);
            let p_base = z1 + dir * a;

            // Dos posibles centros
            for signo in [-1.0, 1.0] {
                let centro = p_base + perp * h * signo;

                // Verificar que es tangente al tercer círculo
                let d3 = (centro - z3).length();
                let r3p = c3.r + r4;
                let r3m = (c3.r - r4).abs();
                // Aceptamos si es tangente exterior o interior
                if (d3 - r3p).abs() < 1e-3 || (d3 - r3m).abs() < 1e-3 {
                    // Verificar que no está duplicado con los existentes (simplificado)
                    // Nos quedamos con el de mayor curvatura (más pequeño)
                    if mejor.is_none() || r4 < mejor.unwrap().r {
                        mejor = Some(Circulo::nuevo(centro.x, centro.y, r4));
                    }
                }
            }
        }
        mejor
    }

    /// Genera la lista de círculos recursivamente (DFS).
    fn generar(
        c1: &Circulo,
        c2: &Circulo,
        c3: &Circulo,
        profundidad: u32,
        max_prof: u32,
        salida: &mut Vec<Circulo>,
    ) {
        if profundidad >= max_prof {
            return;
        }
        if let Some(c4) = Self::cuarto_circulo(c1, c2, c3) {
            // Solo añadimos si es visible (radio > 0.5 px aproximadamente)
            // El corte real se hace al dibujar según el zoom
            salida.push(c4);

            // Tres nuevas ramas: (c1,c2,c4), (c2,c3,c4), (c3,c1,c4)
            Self::generar(c1, c2, &c4, profundidad + 1, max_prof, salida);
            Self::generar(c2, c3, &c4, profundidad + 1, max_prof, salida);
            Self::generar(c3, c1, &c4, profundidad + 1, max_prof, salida);
        }
    }
}

impl Fractal for Apolonio {
    fn nombre(&self) -> &'static str { "Tamiz de Apolonio" }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=8)).changed() {
                self.firma_geometria = None;
                self.circulos_generados.clear();
                self.animando = true;
                let t = ui.input(|i| i.time);
                self.tiempo_inicio = t;
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.circulos_por_segundo, 5.0..=100.0));
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

        // Regenerar geometría solo si cambió la profundidad o el zoom
        let firma = (self.profundidad, self.camara.zoom);
        if self.firma_geometria != Some(firma) {
            self.circulos_generados.clear();
            
            // Configuración: círculo exterior de radio 2 en (0,0)
            let exterior = Circulo::nuevo_encerrando(0.0, 0.0, 2.0);
            // Tres círculos de radio 0.5 tangentes entre sí, formando un triángulo
            let r_int = 0.5;
            let h = r_int * 3.0_f32.sqrt() / 2.0; // altura del triángulo equilátero
            let c1 = Circulo::nuevo(0.0, -h * 2.0 / 3.0, r_int); // abajo
            let c2 = Circulo::nuevo(-r_int, h / 3.0, r_int); // izquierda
            let c3 = Circulo::nuevo(r_int, h / 3.0, r_int); // derecha
            
            // El círculo exterior debe ser tangente a los tres. Ajustamos su radio.
            // Distancia del centro a c1: 2h/3 = 0.577. Radio exterior = 0.577 + 0.5 = 1.077
            let r_ext = (h * 2.0 / 3.0) + r_int;
            let exterior = Circulo::nuevo_encerrando(0.0, 0.0, r_ext);

            // Generar recursivamente
            Self::generar(&exterior, &c1, &c2, 0, self.profundidad, &mut self.circulos_generados);
            Self::generar(&exterior, &c2, &c3, 0, self.profundidad, &mut self.circulos_generados);
            Self::generar(&exterior, &c3, &c1, 0, self.profundidad, &mut self.circulos_generados);

            self.firma_geometria = Some(firma);
        }

        // Animación: dibujar los círculos progresivamente
        let objetivo = if self.animando {
            let ahora = ui.input(|i| i.time);
            let n = ((ahora - self.tiempo_inicio) as f32 * self.circulos_por_segundo) as usize;
            if n >= self.circulos_generados.len() {
                self.animando = false;
                usize::MAX
            } else {
                ui.ctx().request_repaint();
                n
            }
        } else {
            usize::MAX
        };

        let painter = ui.painter();
        let escala = rect.width().min(rect.height()) * 0.4 * self.camara.zoom;
        let cx = centro.x + self.camara.offset.x;
        let cy = centro.y + self.camara.offset.y;

        let mut dibujados = 0;
        for c in &self.circulos_generados {
            if dibujados >= objetivo { break; }
            // Convertir coordenadas del mundo a pantalla
            let px = cx + c.x * escala;
            let py = cy + c.y * escala;
            let pr = c.r * escala;

            // Corte por tamaño: si el círculo es menor que 0.5 px, no lo dibujamos
            if pr < 0.5 { continue; }

            painter.circle_stroke(
                egui::pos2(px, py),
                pr,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 220, 255)),
            );
            dibujados += 1;
        }
    }
}
