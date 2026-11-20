use super::Fractal;
use crate::dibujo::{self, Lote};
use crate::escena::Escena;
use crate::hilos::{self, Reparto};
use eframe::egui::{self, Color32, Pos2};
use rayon::prelude::*;

/// Tamaño del romanesco en unidades del mundo (pensado para una ventana de ~1000 px de alto).
pub const RADIO_BASE: f32 = 370.0;
pub const BASE_Y: f32 = 264.0;

const ANGULO_DORADO: f32 = 2.399_963_2;
const NIVELES: usize = 8;
const FIB: [u32; NIVELES] = [5, 8, 13, 21, 21, 13, 8, 5];
const RADIO_MINIMO: f32 = 1.5;
/// 1.0 = niveles tangentes, <1.0 = se solapan, >1.0 = separados.
const SOLAPE_Y: f32 = 0.7;

#[derive(Clone, Copy)]
pub struct Bulto {
    pub pos: Pos2,
    pub radio: f32,
    pub color: Color32,
    pub hilo: u8,
    /// Instante (en unidades de trabajo) en que su hilo lo termina.
    pub tiempo: f32,
}

fn anillo(nivel: usize, radio_base: f32) -> (f32, f32, u32) {
    let t = nivel as f32 / (NIVELES as f32 - 1.0).max(1.0);
    let radio_nivel = radio_base * (1.0 - t);

    let fib = FIB[nivel.min(FIB.len() - 1)];
    let cap = (radio_nivel * std::f32::consts::PI / RADIO_MINIMO) as u32;
    let num_bultos = fib.min(cap).max(1);

    let radio_bulto = (std::f32::consts::PI * radio_nivel / num_bultos as f32) * 1.05;
    (radio_nivel, radio_bulto, num_bultos)
}

fn alturas_niveles(radio_base: f32) -> [f32; NIVELES] {
    let mut ys = [0.0f32; NIVELES];
    let mut cum = 0.0f32;
    for nivel in 0..NIVELES {
        ys[nivel] = cum;
        let (_, radio_bulto, _) = anillo(nivel, radio_base);
        cum += radio_bulto * SOLAPE_Y;
    }
    ys
}

/// Los bultos de UNA torre (sus 8 anillos), sin recursión.
pub fn hijos(centro: Pos2, radio_base: f32) -> Vec<Bulto> {
    let mut out = Vec::new();
    if radio_base < RADIO_MINIMO {
        return out;
    }
    let ys = alturas_niveles(radio_base);

    for nivel in 0..NIVELES {
        let t = nivel as f32 / (NIVELES as f32 - 1.0).max(1.0);
        let y = centro.y - ys[nivel];
        let (radio_nivel, radio_bulto, num_bultos) = anillo(nivel, radio_base);
        if radio_bulto < RADIO_MINIMO {
            continue;
        }

        let offset_nivel = nivel as f32 * ANGULO_DORADO * 0.5;
        let color = Color32::from_rgb(
            (40.0 + 100.0 * t) as u8,
            (110.0 + 100.0 * t) as u8,
            (30.0 + 60.0 * t) as u8,
        );

        for i in 0..num_bultos {
            let angulo = i as f32 * ANGULO_DORADO + offset_nivel;
            let offset_x = angulo.cos() * radio_nivel;
            let offset_z = angulo.sin() * radio_nivel * 0.35;
            out.push(Bulto {
                pos: egui::pos2(centro.x + offset_x, y + offset_z),
                radio: radio_bulto,
                color,
                hilo: 0,
                tiempo: 0.0,
            });
        }
    }
    out
}

/// Preorden: el bulto y, recursivamente, su propia torre de bultos más pequeños.
pub fn expandir(out: &mut Vec<Bulto>, b: Bulto, profundidad: u32) {
    out.push(b);
    if profundidad > 1 && b.radio > 3.0 {
        for h in hijos(b.pos, b.radio * 0.85) {
            expandir(out, h, profundidad - 1);
        }
    }
}

/// Dibujo de los bultos (la cámara ya se procesó en `Escena::preparar`).
pub fn dibujar_bultos(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    escena: &mut Escena,
    bultos: &[Bulto],
    reparto: &Reparto,
    por_hilo: bool,
    titulo: &str,
) {
    let centro = rect.center();
    let total = reparto.total.ceil() as usize;
    let n = escena.avance(ui, total);
    let hasta = if escena.anim.animando { n as f32 } else { f32::INFINITY };

    let origen = escena.camara.origen(centro);
    let z = escena.camara.zoom;
    let paleta: Vec<Color32> = (0..reparto.tiempos.len()).map(hilos::color_hilo).collect();
    let borde = Color32::from_rgb(20, 60, 20);

    let mut lote = Lote::nuevo();
    for b in bultos {
        if b.tiempo > hasta {
            continue;
        }
        let r = b.radio * z;
        if r < 0.5 {
            continue; // LOD: invisible
        }
        let pos = origen + b.pos.to_vec2() * z;
        if !rect.expand(r).contains(pos) {
            continue; // culling
        }

        let color = if por_hilo {
            hilos::mezclar(b.color, paleta[b.hilo as usize % paleta.len()], 0.7)
        } else {
            b.color
        };

        if r > 5.0 {
            dibujo::disco(&mut lote.mesh, pos, r, borde);
            dibujo::disco(&mut lote.mesh, pos, r - 0.9, color);
        } else {
            dibujo::disco(&mut lote.mesh, pos, r, color);
        }

        lote.revisar();
        if lote.lleno() {
            break;
        }
    }

    let painter = ui.painter_at(rect);
    lote.pintar(&painter);
    hilos::panel(&painter, rect, titulo, &reparto.filas(hasta));
}

pub struct Romanesco {
    escena: Escena,
    paralelo: bool,
    por_hilo: bool,
    bultos: Vec<Bulto>,
    reparto: Reparto,
}

impl Default for Romanesco {
    fn default() -> Self {
        Self {
            escena: Escena::new(2, 400.0),
            paralelo: false,
            por_hilo: true,
            bultos: Vec::new(),
            reparto: Reparto::default(),
        }
    }
}

impl Romanesco {
    /// SECUENCIAL: un solo hilo recorre todo en preorden.
    fn generar_secuencial(&mut self) {
        let mut v = Vec::new();
        for b in hijos(egui::pos2(0.0, BASE_Y), RADIO_BASE) {
            expandir(&mut v, b, self.escena.profundidad);
        }

        // El instante de cada bulto es su posición en el preorden.
        let mut reparto = Reparto::nuevo(1);
        for (i, b) in v.iter_mut().enumerate() {
            b.hilo = 0;
            b.tiempo = (i + 1) as f32;
            reparto.registrar(0, b.tiempo);
        }

        self.bultos = v;
        self.reparto = reparto;
    }

    /// PARALELO: cada "cabeza" del primer anillo es una tarea independiente (ella y todo su
    /// subárbol de cabezas más pequeñas). Se calculan en paralelo con rayon y luego se
    /// reparten entre los trabajadores con una cola dinámica: cada tarea va al hilo
    /// menos cargado, así que las cabezas grandes y las pequeñas se compensan.
    fn generar_paralelo(&mut self) {
        let w = hilos::trabajadores();
        let prof = self.escena.profundidad;

        let raices = hijos(egui::pos2(0.0, BASE_Y), RADIO_BASE);
        let subs: Vec<Vec<Bulto>> = raices
            .into_par_iter()
            .map(|b| {
                let mut v = Vec::new();
                expandir(&mut v, b, prof);
                v
            })
            .collect();

        let costes: Vec<usize> = subs.iter().map(|s| s.len()).collect();
        let plan = hilos::planificar(&costes, w);

        let total: usize = costes.iter().sum();
        let mut todos = Vec::with_capacity(total);
        let mut reparto = Reparto::nuevo(w);
        for (mut sub, &(hilo, inicio)) in subs.into_iter().zip(plan.iter()) {
            for (j, b) in sub.iter_mut().enumerate() {
                b.hilo = hilo as u8;
                b.tiempo = (inicio + j + 1) as f32;
                reparto.registrar(hilo, b.tiempo);
            }
            todos.extend(sub);
        }

        self.bultos = todos;
        self.reparto = reparto;
    }
}

impl Fractal for Romanesco {
    fn nombre(&self) -> &'static str {
        "Brócoli Romanesco"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        let por_hilo = &mut self.por_hilo;
        self.escena.controles(ui, 3, &mut self.paralelo, |ui, paralelo| {
            ui.add_enabled(paralelo, egui::Checkbox::new(por_hilo, "Colorear por hilo"));
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        if self.escena.preparar(ui, rect, self.paralelo) {
            if self.paralelo {
                self.generar_paralelo();
            } else {
                self.generar_secuencial();
            }
        }

        let titulo = if self.paralelo {
            format!("Paralelo: {} hilos", self.reparto.tiempos.len())
        } else {
            "Secuencial: 1 hilo".to_string()
        };
        dibujar_bultos(
            ui,
            rect,
            &mut self.escena,
            &self.bultos,
            &self.reparto,
            self.paralelo && self.por_hilo,
            &titulo,
        );
    }
}
