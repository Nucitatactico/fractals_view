use super::arbol_pitagoras::{
    base_raiz, crear_nodo, dibujar_arbol, generar_rec, Geometria, A45, ANGULO_INICIAL, LADO_MIN,
    LADO_RAIZ, R,
};
use super::Fractal;
use crate::animacion::Animacion;
use crate::camara::Camara;
use crate::dibujo::Cuadrado;
use crate::hilos;
use eframe::egui::{self, Pos2};

pub struct ArbolPitagorasParalelo {
    pub profundidad: u32,
    /// Valor del slider; se aplica al soltarlo (así arrastrar no regenera todo).
    pedida: u32,
    pub camara: Camara,
    anim: Animacion,
    geom: Geometria,
    trabajadores: usize,
    por_hilo: bool,
    firma: Option<u32>,
}

impl Default for ArbolPitagorasParalelo {
    fn default() -> Self {
        Self {
            profundidad: 10,
            pedida: 10,
            camara: Camara::default(),
            anim: Animacion::new(200.0),
            geom: Geometria::default(),
            trabajadores: hilos::trabajadores(),
            por_hilo: true,
            firma: None,
        }
    }
}

/// Fork-join: mientras al subárbol le queden varios trabajadores, se bifurca con
/// `rayon::join` y los trabajadores se reparten entre las dos ramas (mitad y mitad).
/// Cuando queda uno solo, ese hilo termina el subárbol de forma secuencial.
/// `ini..ini+n` son los trabajadores disponibles; el nodo lo atiende `ini`.
fn generar_par(
    base: Pos2,
    lado: f32,
    angulo: f32,
    nivel: u32,
    prof_max: u32,
    ini: usize,
    n: usize,
) -> Vec<Cuadrado> {
    if nivel >= prof_max || lado < LADO_MIN {
        return Vec::new();
    }
    if n <= 1 {
        let mut v = Vec::new();
        generar_rec(&mut v, base, lado, angulo, nivel, prof_max);
        for c in &mut v {
            c.hilo = ini as u8;
        }
        return v;
    }

    let mut nodo = crear_nodo(base, lado, angulo, nivel, prof_max);
    nodo.cuadrado.hilo = ini as u8;
    let l = lado * R;
    let (base_izq, base_der) = (nodo.base_izq, nodo.base_der);
    let n_izq = (n + 1) / 2;
    let n_der = n - n_izq;

    let (izq, der) = rayon::join(
        || generar_par(base_izq, l, angulo + A45, nivel + 1, prof_max, ini, n_izq),
        || generar_par(base_der, l, angulo - A45, nivel + 1, prof_max, ini + n_izq, n_der),
    );

    let mut v = Vec::with_capacity(1 + izq.len() + der.len());
    v.push(nodo.cuadrado);
    v.extend(izq);
    v.extend(der);
    v
}

impl ArbolPitagorasParalelo {
    fn generar(&mut self) {
        let w = hilos::trabajadores();
        let v = generar_par(
            base_raiz(),
            LADO_RAIZ,
            ANGULO_INICIAL,
            0,
            self.profundidad,
            0,
            w,
        );
        self.trabajadores = w;
        self.geom = Geometria::nueva(v, w);
        self.firma = Some(self.profundidad);
    }
}

impl Fractal for ArbolPitagorasParalelo {
    fn nombre(&self) -> &'static str {
        "Árbol de Pitágoras (Simulación Fork-Join)"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            let r = ui.add(egui::Slider::new(&mut self.pedida, 1..=22));
            if r.drag_stopped() || (r.changed() && !r.dragged()) {
                self.profundidad = self.pedida;
                self.anim.reiniciar();
            }
            ui.label("Velocidad (por hilo):");
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 10.0..=5000000.0).logarithmic(true));
            ui.checkbox(&mut self.por_hilo, "Colorear por hilo");
            if ui.button("Repetir animación").clicked() {
                self.anim.reiniciar();
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
        });
    }

    fn dibujar(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        self.camara.interactuar(ui, rect);

        if self.firma != Some(self.profundidad) {
            self.generar();
        }

        let titulo = format!("Paralelo: {} hilos", self.trabajadores);
        dibujar_arbol(
            ui,
            rect,
            &self.camara,
            &mut self.anim,
            &self.geom,
            self.por_hilo,
            &titulo,
        );
    }
}
