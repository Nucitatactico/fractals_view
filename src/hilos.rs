//! Utilidades para visualizar el reparto de trabajo entre "hilos":
//! colores, planificador y panel de progreso por hilo.

use eframe::egui::{self, Align2, Color32, FontId, Painter, Rect};

/// Número de trabajadores que se muestran (hilos del pool de rayon, entre 2 y 12).
pub fn trabajadores() -> usize {
    rayon::current_num_threads().clamp(2, 12)
}

/// Color distinto y estable para cada hilo (ángulo áureo sobre el tono).
pub fn color_hilo(hilo: usize) -> Color32 {
    let h = (0.08 + hilo as f32 * 0.618_034).fract();
    Color32::from(egui::ecolor::Hsva::new(h, 0.70, 0.95, 1.0))
}

pub fn mezclar(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

/// Reparto dinámico: cada tarea va al trabajador con menos carga acumulada
/// (como la cola de trabajo de un pool). Devuelve (trabajador, instante de inicio) por tarea.
pub fn planificar(costes: &[usize], w: usize) -> Vec<(usize, usize)> {
    let mut carga = vec![0usize; w.max(1)];
    costes
        .iter()
        .map(|&c| {
            let i = carga
                .iter()
                .enumerate()
                .min_by_key(|&(_, &l)| l)
                .map(|(i, _)| i)
                .unwrap_or(0);
            let inicio = carga[i];
            carga[i] += c;
            (i, inicio)
        })
        .collect()
}

pub struct FilaHilo {
    pub hilo: usize,
    pub progreso: f32,
}

/// Instante (en unidades de trabajo) en que cada hilo termina cada uno de sus elementos.
pub struct Reparto {
    pub tiempos: Vec<Vec<f32>>,
    pub total: f32,
}

impl Reparto {
    pub fn nuevo(w: usize) -> Self {
        Self { tiempos: vec![Vec::new(); w.max(1)], total: 0.0 }
    }

    /// Hay que registrar los elementos de cada hilo en orden temporal creciente.
    pub fn registrar(&mut self, hilo: usize, t: f32) {
        self.tiempos[hilo].push(t);
        if t > self.total {
            self.total = t;
        }
    }

    /// Solo los hilos que ya nacieron (su primer elemento ya se terminó), en orden de
    /// nacimiento: las barras van apareciendo conforme el fork-join crea hilos.
    pub fn filas(&self, hasta: f32) -> Vec<FilaHilo> {
        let mut filas: Vec<(f32, FilaHilo)> = self
            .tiempos
            .iter()
            .enumerate()
            .filter_map(|(h, v)| {
                let nacimiento = *v.first()?;
                if nacimiento > hasta {
                    return None;
                }
                let hechos = v.partition_point(|&t| t <= hasta);
                Some((
                    nacimiento,
                    FilaHilo { hilo: h, progreso: hechos as f32 / v.len() as f32 },
                ))
            })
            .collect();
        filas.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.hilo.cmp(&b.1.hilo)));
        filas.into_iter().map(|(_, f)| f).collect()
    }
}

impl Default for Reparto {
    fn default() -> Self {
        Self::nuevo(1)
    }
}

/// Panel con una barra de progreso por hilo, en la esquina superior izquierda.
pub fn panel(painter: &Painter, rect: Rect, titulo: &str, filas: &[FilaHilo]) {
    if filas.is_empty() {
        return;
    }
    let alto_fila = 14.0_f32;
    let ancho = 176.0_f32;
    let alto = 28.0 + alto_fila * filas.len() as f32;
    let area = Rect::from_min_size(rect.min + egui::vec2(10.0, 10.0), egui::vec2(ancho, alto));

    painter.rect_filled(area, 6.0_f32, Color32::from_black_alpha(160));
    painter.text(
        area.min + egui::vec2(8.0, 13.0),
        Align2::LEFT_CENTER,
        titulo,
        FontId::proportional(13.0),
        Color32::WHITE,
    );

    for (i, f) in filas.iter().enumerate() {
        let y = area.min.y + 28.0 + i as f32 * alto_fila;
        painter.text(
            egui::pos2(area.min.x + 8.0, y + 6.0),
            Align2::LEFT_CENTER,
            format!("H{}", f.hilo),
            FontId::monospace(10.0),
            Color32::LIGHT_GRAY,
        );
        let barra = Rect::from_min_size(egui::pos2(area.min.x + 40.0, y + 2.0), egui::vec2(126.0, 8.0));
        painter.rect_filled(barra, 2.0_f32, Color32::from_gray(55));
        let lleno = Rect::from_min_size(
            barra.min,
            egui::vec2(barra.width() * f.progreso.clamp(0.0, 1.0), 8.0),
        );
        painter.rect_filled(lleno, 2.0_f32, color_hilo(f.hilo));
    }
}
