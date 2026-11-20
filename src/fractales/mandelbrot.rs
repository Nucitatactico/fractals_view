//! Conjunto de Mandelbrot que se calcula "en vivo": cada frame los trabajadores
//! resuelven unas cuantas filas reales y la imagen va apareciendo.
//! - Secuencial: 1 trabajador recorre la imagen de arriba abajo.
//! - Paralelo: varios trabajadores (rayon) avanzan a la vez, ya sea en bandas contiguas
//!   o con filas intercaladas (así se ve el efecto del balanceo de carga: las filas del
//!   centro cuestan mucho más que las de los bordes).

use super::Fractal;
use crate::hilos::{self, FilaHilo};
use eframe::egui::{self, Color32, ColorImage, Rect, Stroke, TextureHandle, TextureOptions};
use rayon::prelude::*;

const TAM_PALETA: usize = 1024;
const LN2: f64 = std::f64::consts::LN_2;

/// `escala` = unidades del plano complejo por píxel de pantalla.
#[derive(Clone, Copy, PartialEq)]
struct Vista {
    cx: f64,
    cy: f64,
    escala: f64,
}

/// Todo lo que, si cambia, obliga a recalcular la imagen desde cero.
#[derive(Clone, Copy, PartialEq)]
struct Firma {
    w: usize,
    h: usize,
    vista: Vista,
    max_iter: u32,
    bloque: u32,
    paralelo: bool,
    intercalado: bool,
}

#[derive(Clone, Copy)]
struct Params {
    w: usize,
    h: usize,
    cx: f64,
    cy: f64,
    /// Unidades complejas por píxel de la imagen (escala * tamaño de bloque).
    paso: f64,
    max_iter: u32,
}

struct Trabajador {
    filas: Vec<usize>,
    cursor: usize,
}

fn crear_paleta() -> Vec<[u8; 3]> {
    (0..TAM_PALETA)
        .map(|i| {
            let t = i as f32 / TAM_PALETA as f32;
            let c = |fase: f32| {
                ((0.5 + 0.5 * (std::f32::consts::TAU * (t + fase)).cos()) * 255.0) as u8
            };
            [c(0.0), c(0.33), c(0.67)]
        })
        .collect()
}

/// Iteraciones "suaves" hasta escapar, o -1.0 si el punto no escapa (pertenece al conjunto).
fn iterar(cr: f64, ci: f64, max_iter: u32) -> f64 {
    // Atajos: cardioide principal y bulbo de periodo 2 están dentro del conjunto.
    let q = (cr - 0.25) * (cr - 0.25) + ci * ci;
    if q * (q + (cr - 0.25)) <= 0.25 * ci * ci {
        return -1.0;
    }
    if (cr + 1.0) * (cr + 1.0) + ci * ci <= 0.0625 {
        return -1.0;
    }

    let (mut zr, mut zi) = (0.0f64, 0.0f64);
    let (mut zr2, mut zi2) = (0.0f64, 0.0f64);
    let mut i = 0u32;
    // Radio de escape 16 (|z|² > 256): da un degradado de color más suave.
    while i < max_iter && zr2 + zi2 <= 256.0 {
        zi = 2.0 * zr * zi + ci;
        zr = zr2 - zi2 + cr;
        zr2 = zr * zr;
        zi2 = zi * zi;
        i += 1;
    }
    if i >= max_iter {
        -1.0
    } else {
        let log_zn = (zr2 + zi2).ln() * 0.5;
        let nu = log_zn.ln() / LN2;
        i as f64 + 1.0 - nu
    }
}

fn calcular_fila(y: usize, p: &Params, paleta: &[[u8; 3]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(p.w * 4);
    let ci = p.cy + (y as f64 + 0.5 - p.h as f64 / 2.0) * p.paso;
    for x in 0..p.w {
        let cr = p.cx + (x as f64 + 0.5 - p.w as f64 / 2.0) * p.paso;
        let mu = iterar(cr, ci, p.max_iter);
        let rgb = if mu < 0.0 {
            [0, 0, 0]
        } else {
            paleta[((mu / 32.0).fract() * TAM_PALETA as f64) as usize % TAM_PALETA]
        };
        out.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    out
}

fn avanzar_trabajador(
    t: &mut Trabajador,
    filas_por_frame: usize,
    p: &Params,
    paleta: &[[u8; 3]],
) -> Vec<(usize, Vec<u8>)> {
    let hasta = (t.cursor + filas_por_frame).min(t.filas.len());
    let hechas = t.filas[t.cursor..hasta]
        .iter()
        .map(|&y| (y, calcular_fila(y, p, paleta)))
        .collect();
    t.cursor = hasta;
    hechas
}

pub struct Mandelbrot {
    max_iter: u32,
    bloque: u32,
    filas_por_frame: usize,
    paralelo: bool,
    intercalado: bool,

    vista: Option<Vista>,
    firma: Option<Firma>,
    trabajadores: Vec<Trabajador>,
    w: usize,
    h: usize,
    rgba: Vec<u8>,
    textura: Option<TextureHandle>,
    paleta: Vec<[u8; 3]>,
    sucio: bool,
    acabado: bool,
}

impl Default for Mandelbrot {
    fn default() -> Self {
        Self {
            max_iter: 300,
            bloque: 2,
            filas_por_frame: 1,
            paralelo: false,
            intercalado: false,
            vista: None,
            firma: None,
            trabajadores: Vec::new(),
            w: 1,
            h: 1,
            rgba: Vec::new(),
            textura: None,
            paleta: crear_paleta(),
            sucio: false,
            acabado: false,
        }
    }
}

impl Mandelbrot {
    fn reiniciar(&mut self, firma: Firma) {
        self.w = firma.w;
        self.h = firma.h;
        self.rgba = vec![0u8; self.w * self.h * 4];
        for px in self.rgba.chunks_exact_mut(4) {
            px.copy_from_slice(&[18, 18, 22, 255]);
        }

        let nw = if firma.paralelo { hilos::trabajadores() } else { 1 };
        let h = self.h;
        self.trabajadores = (0..nw)
            .map(|k| {
                let filas: Vec<usize> = if nw == 1 {
                    (0..h).collect()
                } else if firma.intercalado {
                    (k..h).step_by(nw).collect()
                } else {
                    let tam = (h + nw - 1) / nw;
                    ((k * tam).min(h)..((k + 1) * tam).min(h)).collect()
                };
                Trabajador { filas, cursor: 0 }
            })
            .collect();

        self.textura = None; // se recrea con el nuevo tamaño
        self.sucio = true;
        self.acabado = false;
        self.firma = Some(firma);
    }

    /// Cada trabajador resuelve hasta `filas_por_frame` filas reales.
    /// En paralelo lo hacen a la vez (rayon); en secuencial es un único recorrido.
    fn avanzar(&mut self) {
        if self.acabado {
            return;
        }
        let Some(firma) = self.firma else { return };
        let p = Params {
            w: self.w,
            h: self.h,
            cx: firma.vista.cx,
            cy: firma.vista.cy,
            paso: firma.vista.escala * firma.bloque as f64,
            max_iter: firma.max_iter,
        };
        let l = self.filas_por_frame.max(1);
        let paleta = &self.paleta;

        let resultados: Vec<Vec<(usize, Vec<u8>)>> = if firma.paralelo {
            self.trabajadores
                .par_iter_mut()
                .map(|t| avanzar_trabajador(t, l, &p, paleta))
                .collect()
        } else {
            self.trabajadores
                .iter_mut()
                .map(|t| avanzar_trabajador(t, l, &p, paleta))
                .collect()
        };

        for filas in resultados {
            for (y, datos) in filas {
                let ini = y * self.w * 4;
                self.rgba[ini..ini + datos.len()].copy_from_slice(&datos);
                self.sucio = true;
            }
        }
        self.acabado = self
            .trabajadores
            .iter()
            .all(|t| t.cursor >= t.filas.len());
    }
}

impl Fractal for Mandelbrot {
    fn nombre(&self) -> &'static str {
        "Conjunto de Mandelbrot"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Iteraciones:");
            ui.add(egui::Slider::new(&mut self.max_iter, 50..=5000).logarithmic(true));
            ui.label("Tamaño de píxel:");
            ui.add(egui::Slider::new(&mut self.bloque, 1..=8));
            ui.label("Filas por frame (por hilo):");
            ui.add(egui::Slider::new(&mut self.filas_por_frame, 1..=64));
            ui.checkbox(&mut self.paralelo, "Paralelo (rayon)");
            ui.add_enabled(
                self.paralelo,
                egui::Checkbox::new(&mut self.intercalado, "Filas intercaladas"),
            );
            if ui.button("Repetir animación").clicked() {
                self.firma = None;
            }
            if ui.button("Reset zoom").clicked() {
                self.vista = None;
            }
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let bloque = self.bloque.max(1) as usize;
        let w = ((rect.width() as usize) / bloque).max(1);
        let h = ((rect.height() as usize) / bloque).max(1);
        let dest = Rect::from_min_size(
            rect.min,
            egui::vec2((w * bloque) as f32, (h * bloque) as f32),
        );

        // Interacción: arrastrar mueve la vista, la rueda hace zoom hacia el cursor.
        let resp = ui.allocate_rect(rect, egui::Sense::drag());
        let v = self.vista.get_or_insert_with(|| Vista {
            cx: -0.5,
            cy: 0.0,
            escala: 2.8 / rect.height().max(1.0) as f64,
        });
        if resp.dragged() {
            let d = resp.drag_delta();
            v.cx -= d.x as f64 * v.escala;
            v.cy -= d.y as f64 * v.escala;
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                if let Some(cursor) = resp.hover_pos() {
                    let factor = (scroll as f64 * 0.002).exp(); // > 1: acercar
                    let off = cursor - dest.center();
                    let nueva = (v.escala / factor).clamp(1e-13, 0.05);
                    v.cx += off.x as f64 * (v.escala - nueva);
                    v.cy += off.y as f64 * (v.escala - nueva);
                    v.escala = nueva;
                }
            }
        }
        let vista = *v;

        // Si cambió algo (vista, tamaño, parámetros) se vuelve a empezar.
        let firma = Firma {
            w,
            h,
            vista,
            max_iter: self.max_iter,
            bloque: self.bloque.max(1),
            paralelo: self.paralelo,
            intercalado: self.intercalado,
        };
        if self.firma != Some(firma) {
            self.reiniciar(firma);
        }

        self.avanzar();

        if self.sucio {
            let img = ColorImage::from_rgba_unmultiplied([self.w, self.h], &self.rgba);
            if let Some(t) = self.textura.as_mut() {
                t.set(img, TextureOptions::NEAREST);
            } else {
                self.textura = Some(ui.ctx().load_texture("mandelbrot", img, TextureOptions::NEAREST));
            }
            self.sucio = false;
        }
        if !self.acabado {
            ui.ctx().request_repaint();
        }

        let painter = ui.painter_at(rect);
        if let Some(t) = &self.textura {
            painter.image(
                t.id(),
                dest,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        // Una línea de color por trabajador marca la fila que está resolviendo.
        if !self.acabado {
            for (k, t) in self.trabajadores.iter().enumerate() {
                if t.cursor < t.filas.len() {
                    let y = dest.min.y + (t.filas[t.cursor] as f32 + 0.5) * bloque as f32;
                    painter.line_segment(
                        [egui::pos2(dest.min.x, y), egui::pos2(dest.max.x, y)],
                        Stroke::new(2.0_f32, hilos::color_hilo(k)),
                    );
                }
            }
        }

        let filas: Vec<FilaHilo> = self
            .trabajadores
            .iter()
            .enumerate()
            .map(|(k, t)| FilaHilo {
                hilo: k,
                progreso: if t.filas.is_empty() {
                    1.0
                } else {
                    t.cursor as f32 / t.filas.len() as f32
                },
            })
            .collect();
        let titulo = if self.trabajadores.len() > 1 {
            format!("Paralelo: {} hilos", self.trabajadores.len())
        } else {
            "Secuencial: 1 hilo".to_string()
        };
        hilos::panel(&painter, rect, &titulo, &filas);
    }
}
