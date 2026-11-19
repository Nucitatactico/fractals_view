//! Rutinas de dibujo por lotes: pocas `Shape` grandes en vez de miles pequeñas.

use crate::camara::Camara;
use eframe::egui::{self, Color32, Mesh, Pos2, Shape, Stroke};

/// Puntos por trozo de polilínea. Al partirla, egui descarta los trozos
/// que quedan completamente fuera del panel.
const TROZO: usize = 2048;
const MAX_VERTICES_MESH: usize = 60_000;

#[derive(Clone, Copy)]
pub struct Cuadrado {
    pub p: [Pos2; 4],
    pub lado: f32,
    pub color: Color32,
    pub nivel: u8,
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
    let origen = camara.origen(centro);
    let z = camara.zoom;
    let mut shapes: Vec<Shape> = Vec::new();
    let mut buf: Vec<Pos2> = Vec::with_capacity(TROZO + 1);

    for p in puntos.iter().step_by(paso.max(1)) {
        let s = origen + p.to_vec2() * z;
        buf.push(s);
        if buf.len() > TROZO {
            let ultimo = s;
            shapes.push(Shape::line(
                std::mem::replace(&mut buf, Vec::with_capacity(TROZO + 1)),
                stroke,
            ));
            buf.push(ultimo); // el siguiente trozo continúa donde terminó este
        }
    }
    if buf.len() > 1 {
        shapes.push(Shape::line(buf, stroke));
    }
    painter.extend(shapes);
}

/// Dibuja cuadrados (coordenadas del mundo) en un solo `Mesh`.
/// Aplica culling (fuera del panel) y LOD (cuadrados < 0.7 px se omiten).
pub fn cuadrados(
    painter: &egui::Painter,
    camara: &Camara,
    centro: Pos2,
    rect: egui::Rect,
    lista: &[Cuadrado],
) {
    let origen = camara.origen(centro);
    let z = camara.zoom;

    let mut mesh = Mesh::default();
    let mut shapes: Vec<Shape> = Vec::new();
    let mut bordes: Vec<Shape> = Vec::new();

    for c in lista {
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

        let i = mesh.vertices.len() as u32;
        for p in pts {
            mesh.colored_vertex(p, c.color);
        }
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i, i + 2, i + 3);

        // El borde solo se nota (y vale la pena) en cuadrados grandes.
        if lado_px > 6.0 {
            bordes.push(Shape::closed_line(
                pts.to_vec(),
                Stroke::new(0.5, Color32::BLACK),
            ));
        }

        if mesh.vertices.len() >= MAX_VERTICES_MESH {
            shapes.push(Shape::mesh(std::mem::take(&mut mesh)));
        }
    }

    if !mesh.is_empty() {
        shapes.push(Shape::mesh(mesh));
    }
    shapes.extend(bordes);
    painter.extend(shapes);
}
