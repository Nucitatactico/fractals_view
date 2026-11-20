//! Curvas (polilíneas) cuyo trabajo se reparte entre "hilos".
//! Secuencial = 1 trabajador con toda la curva; paralelo = bloques contiguos, uno por trabajador.

use crate::camara::Camara;
use crate::dibujo;
use crate::hilos::FilaHilo;
use eframe::egui::{Color32, Painter, Pos2, Stroke};

struct Tramo {
    ini: usize,
    fin: usize,
    hilo: usize,
}

pub struct Curva {
    /// Vértices en coordenadas del mundo (segmentos + 1).
    puntos: Vec<Pos2>,
    tramos: Vec<Tramo>,
    trabajo: Vec<usize>,
    /// Segmentos del trabajador con más carga (define la duración de la animación).
    pub max_trabajo: usize,
}

impl Curva {
    pub fn vacia() -> Self {
        Self::secuencial(Vec::new())
    }

    pub fn secuencial(puntos: Vec<Pos2>) -> Self {
        let n = puntos.len().saturating_sub(1);
        Self {
            puntos,
            tramos: vec![Tramo { ini: 0, fin: n, hilo: 0 }],
            trabajo: vec![n],
            max_trabajo: n,
        }
    }

    /// Divide la curva en `w` bloques contiguos del mismo tamaño.
    pub fn bloques(puntos: Vec<Pos2>, w: usize) -> Self {
        let n = puntos.len().saturating_sub(1);
        let w = w.max(1);
        let tam = ((n + w - 1) / w).max(1);
        let mut tramos = Vec::new();
        let mut trabajo = Vec::new();
        for b in 0..w {
            let ini = b * tam;
            if ini >= n {
                break;
            }
            let fin = ((b + 1) * tam).min(n);
            tramos.push(Tramo { ini, fin, hilo: b });
            trabajo.push(fin - ini);
        }
        let max_trabajo = trabajo.iter().copied().max().unwrap_or(0);
        Self { puntos, tramos, trabajo, max_trabajo }
    }

    /// `n` = segmentos que ya dibujó cada trabajador. Mientras un trabajador sigue
    /// dibujando se marca su posición actual con un punto del color de su hilo.
    pub fn dibujar(
        &self,
        painter: &Painter,
        camara: &Camara,
        centro: Pos2,
        n: usize,
        animando: bool,
        paso: usize,
        grosor: f32,
        color: impl Fn(usize) -> Color32,
    ) {
        for t in &self.tramos {
            let largo = t.fin - t.ini;
            let k = n.min(largo);
            if k == 0 {
                continue;
            }
            let c = color(t.hilo);
            dibujo::polilinea(
                painter,
                camara,
                centro,
                &self.puntos[t.ini..=t.ini + k],
                paso,
                Stroke::new(grosor, c),
            );

            if animando && k < largo {
                let cabeza = camara.origen(centro) + self.puntos[t.ini + k].to_vec2() * camara.zoom;
                painter.circle_filled(cabeza, 3.5, c);
            }
        }
    }

    pub fn filas(&self, n: usize) -> Vec<FilaHilo> {
        self.trabajo
            .iter()
            .enumerate()
            .map(|(h, &t)| FilaHilo {
                hilo: h,
                progreso: if t == 0 { 1.0 } else { (n as f32 / t as f32).min(1.0) },
            })
            .collect()
    }
}
