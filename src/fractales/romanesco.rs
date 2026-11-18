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
            bultos_por_segundo: 400.0,
        }
    }
}

impl Romanesco {
    const ANGULO_DORADO: f32 = 2.399_963_2;
    const NIVELES: usize = 8;
    const FIB: [u32; Self::NIVELES] = [5, 8, 13, 21, 21, 13, 8, 5];
    const RADIO_MINIMO_PX: f32 = 1.5;

    // Factor de solape vertical entre niveles contiguos.
    // 1.0 = tangentes, <1.0 = se solapan (más unido), >1.0 = separados.
    const SOLAPE_Y: f32 = 0.7;

    fn contar_bultos(radio_base: f32, profundidad: u32) -> u32 {
        if profundidad == 0 || radio_base < Self::RADIO_MINIMO_PX {
            return 0;
        }
        let mut total = 0u32;
        for nivel in 0..Self::NIVELES {
            let (_, radio_bulto, num_bultos) = Self::anillo(nivel, radio_base);
            if radio_bulto < Self::RADIO_MINIMO_PX {
                continue;
            }
            for _ in 0..num_bultos {
                total += 1;
                if profundidad > 1 && radio_bulto > 3.0 {
                    total += Self::contar_bultos(radio_bulto * 0.85, profundidad - 1);
                }
            }
        }
        total
    }

    fn anillo(nivel: usize, radio_base: f32) -> (f32, f32, u32) {
        let t = nivel as f32 / (Self::NIVELES as f32 - 1.0).max(1.0);
        let radio_nivel = radio_base * (1.0 - t);

        let fib = Self::FIB[nivel.min(Self::FIB.len() - 1)];
        let cap = (radio_nivel * std::f32::consts::PI / Self::RADIO_MINIMO_PX) as u32;
        let num_bultos = fib.min(cap).max(1);

        let radio_bulto =
            (std::f32::consts::PI * radio_nivel / num_bultos as f32) * 1.05;

        (radio_nivel, radio_bulto, num_bultos)
    }

    /// Devuelve la altura Y acumulada (relativa al centro) de cada nivel.
    /// Se construye apilando el radio real del bulto de cada nivel, así
    /// que los niveles superiores nunca se alejan más de lo que su propio
    /// tamaño permite: no más "islas flotantes".
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

    fn dibujar_torre(
        painter: &egui::Painter,
        centro: egui::Pos2,
        radio_base: f32,
        profundidad: u32,
        visitados: &mut u32,
        objetivo: u32,
    ) -> bool {
        if profundidad == 0 || radio_base < Self::RADIO_MINIMO_PX {
            return true;
        }

        // Y acumulado por nivel (una sola vez por torre)
        let ys = Self::alturas_niveles(radio_base);

        for nivel in 0..Self::NIVELES {
            let t = nivel as f32 / (Self::NIVELES as f32 - 1.0).max(1.0);
            let y = centro.y - ys[nivel];

            let (radio_nivel, radio_bulto, num_bultos) = Self::anillo(nivel, radio_base);

            if radio_bulto < Self::RADIO_MINIMO_PX {
                continue;
            }

            let offset_nivel = nivel as f32 * Self::ANGULO_DORADO * 0.5;

            for i in 0..num_bultos {
                if *visitados >= objetivo {
                    return false;
                }
                *visitados += 1;

                let angulo = i as f32 * Self::ANGULO_DORADO + offset_nivel;
                let offset_x = angulo.cos() * radio_nivel;
                let offset_z = angulo.sin() * radio_nivel * 0.35;
                let pos = egui::pos2(centro.x + offset_x, y + offset_z);

                let color = egui::Color32::from_rgb(
                    (40.0 + 100.0 * t) as u8,
                    (110.0 + 100.0 * t) as u8,
                    (30.0 + 60.0 * t) as u8,
                );

                painter.circle_filled(pos, radio_bulto, color);
                painter.circle_stroke(
                    pos,
                    radio_bulto,
                    egui::Stroke::new(0.6_f32, egui::Color32::from_rgb(20, 60, 20)),
                );

                if profundidad > 1 && radio_bulto > 3.0 {
                    let exito = Self::dibujar_torre(
                        painter,
                        pos,
                        radio_bulto * 0.85,
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
    fn nombre(&self) -> &'static str {
        "Brócoli Romanesco"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            if ui.add(egui::Slider::new(&mut self.profundidad, 1..=3)).changed() {
                let t = ui.input(|i| i.time);
                self.animando = true;
                self.tiempo_inicio = t;
            }
            ui.label("Velocidad:");
            ui.add(egui::Slider::new(&mut self.bultos_por_segundo, 10.0..=2000.0));
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
        self.camara.procesar_zoom(ui, centro);
        self.camara.procesar_pan(ui);

        let altura_base = rect.height() * 0.6 * self.camara.zoom;
        // Antes era 0.35 porque la altura se "estiraba" artificialmente.
        // Ahora el fractal ocupa su tamaño real (determinado por los radios
        // de los bultos), así que conviene subir el radio base.
        let radio_base = altura_base * 0.70;

        let base = egui::pos2(
            centro.x + self.camara.offset.x,
            centro.y + altura_base * 0.5 + self.camara.offset.y,
        );

        let total_bultos = Self::contar_bultos(radio_base, self.profundidad).max(1);

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
            radio_base,
            self.profundidad,
            &mut visitados,
            objetivo,
        );
    }
}
