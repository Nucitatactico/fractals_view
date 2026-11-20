//! Piezas comunes a todos los fractales: detección de cambios (`Firma`) y el
//! "esqueleto" de cada vista (`Escena`: profundidad + cámara + animación).

use crate::animacion::Animacion;
use crate::camara::Camara;
use eframe::egui;

/// Recuerda con qué valores se generó algo y avisa cuando cambian.
pub struct Firma<T: PartialEq> {
    valor: Option<T>,
}

impl<T: PartialEq> Default for Firma<T> {
    fn default() -> Self {
        Self { valor: None }
    }
}

impl<T: PartialEq> Firma<T> {
    /// `true` si `v` es distinto de lo último que se vio (y lo recuerda).
    pub fn cambiada(&mut self, v: T) -> bool {
        if self.valor.as_ref() == Some(&v) {
            false
        } else {
            self.valor = Some(v);
            true
        }
    }

    /// Fuerza que la próxima llamada a `cambiada` devuelva `true`.
    pub fn invalidar(&mut self) {
        self.valor = None;
    }

    pub fn valor(&self) -> Option<&T> {
        self.valor.as_ref()
    }
}

/// Profundidad + cámara + animación + control de cambios: lo que repetían todos los fractales.
pub struct Escena {
    pub profundidad: u32,
    /// Valor del slider; se aplica a `profundidad` al soltarlo (así arrastrar no regenera todo).
    pedida: u32,
    pub camara: Camara,
    pub anim: Animacion,
    firma: Firma<(u32, bool)>,
}

impl Escena {
    pub fn new(profundidad: u32, velocidad: f32) -> Self {
        Self {
            profundidad,
            pedida: profundidad,
            camara: Camara::default(),
            anim: Animacion::new(velocidad),
            firma: Firma::default(),
        }
    }

    /// Fila de controles común: profundidad, modo paralelo, extras del fractal,
    /// velocidad, "Repetir animación" y "Reset zoom".
    pub fn controles(
        &mut self,
        ui: &mut egui::Ui,
        max_profundidad: u32,
        paralelo: &mut bool,
        extra: impl FnOnce(&mut egui::Ui, bool),
    ) {
        ui.horizontal(|ui| {
            ui.label("Profundidad:");
            let r = ui.add(egui::Slider::new(&mut self.pedida, 1..=max_profundidad));
            if r.drag_stopped() || (r.changed() && !r.dragged()) {
                self.profundidad = self.pedida;
            }

            ui.checkbox(paralelo, "Paralelo (rayon)");
            extra(ui, *paralelo);

            ui.label(if *paralelo { "Velocidad (por hilo):" } else { "Velocidad:" });
            ui.add(egui::Slider::new(&mut self.anim.velocidad, 10.0..=5_000_000.0).logarithmic(true));

            if ui.button("Repetir animación").clicked() {
                self.anim.reiniciar();
            }
            if ui.button("Reset zoom").clicked() {
                self.camara.reset();
            }
        });
    }

    /// Procesa pan y zoom y detecta si cambió la profundidad o el modo.
    /// Devuelve `true` si hay que (re)generar la geometría; en ese caso reinicia la animación.
    pub fn preparar(&mut self, ui: &mut egui::Ui, rect: egui::Rect, paralelo: bool) -> bool {
        self.camara.interactuar(ui, rect);
        let cambio = self.firma.cambiada((self.profundidad, paralelo));
        if cambio {
            self.anim.reiniciar();
        }
        cambio
    }

    /// Cuántos elementos (de `total`) se deben mostrar ahora.
    pub fn avance(&mut self, ui: &egui::Ui, total: usize) -> usize {
        self.anim.progreso(ui, total)
    }
}
