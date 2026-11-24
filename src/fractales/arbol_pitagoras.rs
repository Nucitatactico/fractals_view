use super::Fractal;
use crate::dibujo::{self, Cuadrado};
use crate::escena::Escena;
use crate::hilos::{self, Reparto};
use eframe::egui::{self, Color32, Pos2};
use std::f32::consts::{FRAC_1_SQRT_2, FRAC_PI_4, PI};

pub const LADO_RAIZ: f32 = 140.0;
pub const LADO_MIN: f32 = 0.01;
pub const A45: f32 = FRAC_PI_4;
pub const R: f32 = FRAC_1_SQRT_2;
pub const ANGULO_INICIAL: f32 = -PI;

pub struct Nodo {
    pub cuadrado: Cuadrado,
    pub base_izq: Pos2,
    pub base_der: Pos2,
}

/// Calcula el cuadrado de un nodo y las bases de sus dos hijos.
pub fn crear_nodo(base: Pos2, lado: f32, angulo: f32, nivel: u32, prof_max: u32) -> Nodo {
    let (sin, cos) = angulo.sin_cos();
    let v1 = base + egui::vec2(cos * lado, sin * lado);
    let v2 = v1 + egui::vec2(-sin * lado, cos * lado);
    let v3 = base + egui::vec2(-sin * lado, cos * lado);
    // Ápice del triángulo rectángulo sobre el lado superior: ahí nace el hijo derecho.
    let (sin_h, cos_h) = (angulo + A45).sin_cos();
    let apice = v3 + egui::vec2(cos_h * lado * R, sin_h * lado * R);

    // t = 0 en la raíz, t = 1 en las hojas. Café -> verde claro.
    let t = if prof_max > 1 {
        (nivel as f32 / (prof_max - 1) as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let color = Color32::from_rgb(
        (139.0 + (100.0 - 139.0) * t) as u8,
        (90.0 + (200.0 - 90.0) * t) as u8,
        (43.0 + (80.0 - 43.0) * t) as u8,
    );

    Nodo {
        cuadrado: Cuadrado {
            p: [base, v1, v2, v3],
            lado,
            color,
            nivel: nivel.min(255) as u8,
            hilo: 0,
            tiempo: 0.0,
        },
        base_izq: v3,
        base_der: apice,
    }
}

/// Recursión secuencial en preorden (DFS). Escribe en coordenadas del mundo.
pub fn generar_rec(
    out: &mut Vec<Cuadrado>,
    base: Pos2,
    lado: f32,
    angulo: f32,
    nivel: u32,
    prof_max: u32,
) {
    if nivel >= prof_max || lado < LADO_MIN {
        return;
    }
    let n = crear_nodo(base, lado, angulo, nivel, prof_max);
    out.push(n.cuadrado);
    let l = lado * R;
    generar_rec(out, n.base_izq, l, angulo + A45, nivel + 1, prof_max);
    generar_rec(out, n.base_der, l, angulo - A45, nivel + 1, prof_max);
}

pub fn base_raiz() -> Pos2 {
    egui::pos2(LADO_RAIZ / 2.0, 0.0)
}

/// Cuadrados en preorden + el instante en que cada hilo termina cada uno.
pub struct Geometria {
    pub cuadrados: Vec<Cuadrado>,
    pub reparto: Reparto,
}

impl Default for Geometria {
    fn default() -> Self {
        Self { cuadrados: Vec::new(), reparto: Reparto::default() }
    }
}

impl Geometria {
    pub fn nueva(mut cuadrados: Vec<Cuadrado>, trabajadores: usize) -> Self {
        centrar(&mut cuadrados);
        let reparto = asignar_tiempos(&mut cuadrados, trabajadores);
        Self { cuadrados, reparto }
    }
}

/// Centra el árbol en el origen usando la caja que envuelve todos sus vértices.
fn centrar(cuadrados: &mut [Cuadrado]) {
    if cuadrados.is_empty() {
        return;
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for c in cuadrados.iter() {
        for p in &c.p {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
        }
    }
    let d = egui::vec2(-(min_x + max_x) / 2.0, -(min_y + max_y) / 2.0);
    for c in cuadrados.iter_mut() {
        for p in &mut c.p {
            *p += d;
        }
    }
}

/// Simula la ejecución: cada hilo procesa sus nodos uno tras oto
pub fn asignar_tiempos(cuadrados: &mut [Cuadrado], trabajadores: usize) -> Reparto {
    let w = trabajadores.max(1);
    let mut libre = vec![0.0f32; w];
    let mut pila = [0.0f32; 256]; // instante del último ancestro visto en cada nivel
    let mut reparto = Reparto::nuevo(w);

    for c in cuadrados.iter_mut() {
        let h = (c.hilo as usize).min(w - 1);
        let nivel = c.nivel as usize;
        let t_padre = if nivel > 0 { pila[nivel - 1] } else { 0.0 };
        let t = libre[h].max(t_padre) + 1.0;
        libre[h] = t;
        pila[nivel] = t;
        c.tiempo = t;
        reparto.registrar(h, t);
    }
    reparto
}

/// Cuando queda uno solo, ese hilo termina el subárbol de forma secuencial
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

pub fn dibujar_arbol(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    escena: &mut Escena,
    geom: &Geometria,
    por_hilo: bool,
    titulo: &str,
) {
    let centro = rect.center();
    let total = geom.reparto.total.ceil() as usize;
    let n = escena.avance(ui, total);
    let hasta = if escena.anim.animando { n as f32 } else { f32::INFINITY };

    let paleta: Vec<Color32> = (0..geom.reparto.tiempos.len()).map(hilos::color_hilo).collect();
    let painter = ui.painter_at(rect);
    dibujo::cuadrados(
        &painter,
        &escena.camara,
        centro,
        rect,
        &geom.cuadrados,
        hasta,
        if por_hilo { Some(paleta.as_slice()) } else { None },
    );
    hilos::panel(&painter, rect, titulo, &geom.reparto.filas(hasta));
}

pub struct ArbolPitagoras {
    escena: Escena,
    paralelo: bool,
    por_hilo: bool,
    geom: Geometria,
}

impl Default for ArbolPitagoras {
    fn default() -> Self {
        Self {
            escena: Escena::new(10, 200.0),
            paralelo: false,
            por_hilo: true,
            geom: Geometria::default(),
        }
    }
}

impl ArbolPitagoras {
    fn generar_secuencial(&mut self) {
        let mut v = Vec::new();
        generar_rec(
            &mut v,
            base_raiz(),
            LADO_RAIZ,
            ANGULO_INICIAL,
            0,
            self.escena.profundidad,
        );
        self.geom = Geometria::nueva(v, 1);
    }

    fn generar_paralelo(&mut self) {
        let w = hilos::trabajadores();
        let v = generar_par(
            base_raiz(),
            LADO_RAIZ,
            ANGULO_INICIAL,
            0,
            self.escena.profundidad,
            0,
            w,
        );
        self.geom = Geometria::nueva(v, w);
    }
}

impl Fractal for ArbolPitagoras {
    fn nombre(&self) -> &'static str {
        "Árbol de Pitágoras"
    }

    fn ui_controles(&mut self, ui: &mut egui::Ui) {
        let por_hilo = &mut self.por_hilo;
        self.escena.controles(ui, 22, &mut self.paralelo, |ui, paralelo| {
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
            format!("Paralelo: {} hilos", self.geom.reparto.tiempos.len())
        } else {
            "Secuencial: 1 hilo".to_string()
        };
        dibujar_arbol(
            ui,
            rect,
            &mut self.escena,
            &self.geom,
            self.paralelo && self.por_hilo,
            &titulo,
        );
    }
}
impl ArbolPitagoras {
    pub fn medir(profundidad: u32, paralelo: bool) -> usize {
        fn rama_seq(base: egui::Pos2, lado: f32, angulo: f32, nivel: u32, max_nivel: u32, out: &mut Vec<egui::Pos2>) {
            if nivel > max_nivel || lado < LADO_MIN {
                return;
            }
            let dx = lado * angulo.cos();
            let dy = lado * angulo.sin();
            let p2 = egui::Pos2::new(base.x + dx, base.y + dy);
            out.push(p2);

            let lado_izq = lado * R;
            let lado_der = lado * R;
            let ang_izq = angulo - A45;
            let ang_der = angulo + A45;
            let cumbre = egui::Pos2::new(p2.x + lado_izq * ang_izq.cos(), p2.y + lado_izq * ang_izq.sin());

            rama_seq(p2, lado_izq, ang_izq, nivel + 1, max_nivel, out);
            rama_seq(cumbre, lado_der, ang_der, nivel + 1, max_nivel, out);
        }

        let mut puntos = Vec::new();
        let inicio = base_raiz();
        if paralelo {
            let lado_izq = LADO_RAIZ * R;
            let lado_der = LADO_RAIZ * R;
            let ang_izq = ANGULO_INICIAL - A45;
            let ang_der = ANGULO_INICIAL + A45;
            let p2 = egui::Pos2::new(inicio.x + LADO_RAIZ * ANGULO_INICIAL.cos(), inicio.y + LADO_RAIZ * ANGULO_INICIAL.sin());
            let cumbre = egui::Pos2::new(p2.x + lado_izq * ang_izq.cos(), p2.y + lado_izq * ang_izq.sin());

            let (mut izq, mut der) = rayon::join(
                || {
                    let mut v = Vec::new();
                    rama_seq(p2, lado_izq, ang_izq, 1, profundidad, &mut v);
                    v
                },
                || {
                    let mut v = Vec::new();
                    rama_seq(cumbre, lado_der, ang_der, 1, profundidad, &mut v);
                    v
                },
            );
            puntos.push(p2);
            puntos.append(&mut izq);
            puntos.append(&mut der);
        } else {
            rama_seq(inicio, LADO_RAIZ, ANGULO_INICIAL, 0, profundidad, &mut puntos);
        }
        puntos.len()
    }
}
