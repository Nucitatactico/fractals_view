//! Rutinas de dibujo por lotes: pocas `Shape` grandes en vez de miles pequeñas,
//! con culling, nivel de detalle y un tope de vértices por frame.

use crate::camara::Camara;
use crate::hilos;
use eframe::egui::{self, Color32, Mesh, Pos2, Shape, Stroke, Vec2};
use std::sync::OnceLock;

/// Puntos por trozo de polilínea. Al partirla, egui descarta los trozos
/// que quedan completamente fuera del panel.
const TROZO: usize = 2048;
const MAX_PUNTOS_POLILINEA: usize = 400_000;
const MAX_BORDES: usize = 20_000;
pub const MAX_VERTICES_MESH: usize = 60_000;
/// Tope de vértices por frame. El buffer de egui_wgpu crece a potencias de 2:
/// por encima de ~8,4 M vértices pide más de 256 MiB y wgpu aborta.
pub const MAX_VERTICES_FRAME: usize = 3_000_000;

#[derive(Clone, Copy)]
pub struct Cuadrado {
    pub p: [Pos2; 4],
    pub lado: f32,
    pub color: Color32,
    pub nivel: u8,
    pub hilo: u8,
    /// Instante (en unidades de trabajo) en que su hilo lo termina.
    pub tiempo: f32,
}

/// Acumula triángulos en `Mesh`es de tamaño acotado.
pub struct Lote {
    pub mesh: Mesh,
    shapes: Vec<Shape>,
    volcados: usize,
}

impl Lote {
    pub fn nuevo() -> Self {
        Self { mesh: Mesh::default(), shapes: Vec::new(), volcados: 0 }
    }

    /// Llamar tras añadir geometría: parte el mesh cuando se hace grande.
    pub fn revisar(&mut self) {
        if self.mesh.vertices.len() >= MAX_VERTICES_MESH {
            self.volcados += self.mesh.vertices.len();
            self.shapes.push(Shape::mesh(std::mem::take(&mut self.mesh)));
        }
    }

    pub fn lleno(&self) -> bool {
        self.volcados + self.mesh.vertices.len() >= MAX_VERTICES_FRAME
    }

    pub fn pintar(mut self, painter: &egui::Painter) {
        if !self.mesh.is_empty() {
            self.shapes.push(Shape::mesh(self.mesh));
        }
        painter.extend(self.shapes);
    }
}

/// Dibuja una polilínea dada en coordenadas del mundo.
/// `paso` > 1 salta puntos (nivel de detalle) cuando los segmentos miden < 1 px.
pub fn polilinea(
    painter: &egui::Painter,
    camara: &Camara,
    centro: Pos2,
    puntos: &[Pos2],
    paso: usize,
    stroke: Stroke,
) {
    if puntos.len() < 2 {
        return;
    }
    let mut paso = paso.max(1);
    if puntos.len() / paso > MAX_PUNTOS_POLILINEA {
        paso = puntos.len() / MAX_PUNTOS_POLILINEA + 1;
    }
    let origen = camara.origen(centro);
    let z = camara.zoom;
    let ultimo = puntos.len() - 1;

    let mut shapes: Vec<Shape> = Vec::new();
    let mut buf: Vec<Pos2> = Vec::with_capacity(TROZO + 1);
    let mut i = 0usize;
    loop {
        let s = origen + puntos[i].to_vec2() * z;
        buf.push(s);
        if buf.len() > TROZO {
            shapes.push(Shape::line(
                std::mem::replace(&mut buf, Vec::with_capacity(TROZO + 1)),
                stroke,
            ));
            buf.push(s); // el siguiente trozo continúa donde terminó este
        }
        if i == ultimo {
            break;
        }
        i = (i + paso).min(ultimo);
    }
    if buf.len() > 1 {
        shapes.push(Shape::line(buf, stroke));
    }
    painter.extend(shapes);
}

/// Dibuja cuadrados desde hasta donde se pide con base en su hilo
pub fn cuadrados(
    painter: &egui::Painter,
    camara: &Camara,
    centro: Pos2,
    rect: egui::Rect,
    lista: &[Cuadrado],
    hasta: f32,
    paleta: Option<&[Color32]>,
) {
    let origen = camara.origen(centro);
    let z = camara.zoom;

    let mut lote = Lote::nuevo();
    let mut bordes: Vec<Shape> = Vec::new();

    for c in lista {
        if c.tiempo > hasta {
            continue;
        }
        let lado_px = c.lado * z;
        if lado_px < 0.7 {
            continue;
        }
        let p0 = origen + c.p[0].to_vec2() * z;
        if !rect.expand(lado_px * 2.0).contains(p0) {
            continue;
        }
        let pts = [
            p0,
            origen + c.p[1].to_vec2() * z,
            origen + c.p[2].to_vec2() * z,
            origen + c.p[3].to_vec2() * z,
        ];

        let color = match paleta {
            Some(p) if !p.is_empty() => hilos::mezclar(c.color, p[c.hilo as usize % p.len()], 0.8),
            _ => c.color,
        };

        let i = lote.mesh.vertices.len() as u32;
        for p in pts {
            lote.mesh.colored_vertex(p, color);
        }
        lote.mesh.add_triangle(i, i + 1, i + 2);
        lote.mesh.add_triangle(i, i + 2, i + 3);

        if lado_px > 6.0 && bordes.len() < MAX_BORDES {
            bordes.push(Shape::closed_line(
                pts.to_vec(),
                Stroke::new(0.5_f32, Color32::BLACK),
            ));
        }

        lote.revisar();
        if lote.lleno() {
            break;
        }
    }

    lote.pintar(painter);
    painter.extend(bordes);
}

//Dicsos

const SEGMENTOS: [usize; 4] = [4, 8, 14, 24];
static TABLAS: OnceLock<[Vec<Vec2>; 4]> = OnceLock::new();

fn tablas() -> &'static [Vec<Vec2>; 4] {
    TABLAS.get_or_init(|| {
        SEGMENTOS.map(|n| {
            (0..n)
                .map(|k| {
                    let a = k as f32 / n as f32 * std::f32::consts::TAU;
                    egui::vec2(a.cos(), a.sin())
                })
                .collect()
        })
    })
}

/// Agrga un disco relleno al mesh con un número de lados acorde a su tamaño en pantalla.
pub fn disco(mesh: &mut Mesh, centro: Pos2, r: f32, color: Color32) {
    let idx = if r < 2.0 {
        0
    } else if r < 5.0 {
        1
    } else if r < 14.0 {
        2
    } else {
        3
    };
    let unidades = &tablas()[idx];
    let n = unidades.len() as u32;

    let i0 = mesh.vertices.len() as u32;
    mesh.colored_vertex(centro, color);
    for u in unidades {
        mesh.colored_vertex(centro + *u * r, color);
    }
    for k in 0..n {
        mesh.add_triangle(i0, i0 + 1 + k, i0 + 1 + (k + 1) % n);
    }
}
