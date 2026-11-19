use eframe::egui;

/// Reloj de animación compartido por todos los fractales.
/// `velocidad` = elementos (segmentos, nodos, niveles...) por segundo.
pub struct Animacion {
    pub animando: bool,
    pub velocidad: f32,
    inicio: Option<f64>,
}

impl Animacion {
    pub fn new(velocidad: f32) -> Self {
        Self { animando: true, velocidad, inicio: None }
    }

    /// Reinicia la animación. El reloj arranca en el siguiente frame
    /// (así no se "salta" la animación si la app lleva rato abierta).
    pub fn reiniciar(&mut self) {
        self.animando = true;
        self.inicio = None;
    }

    /// Devuelve cuántos elementos (de `total`) se deben mostrar ahora.
    /// Pide repintado solo mientras la animación sigue en curso.
    pub fn progreso(&mut self, ui: &egui::Ui, total: usize) -> usize {
        if !self.animando {
            return total;
        }
        let ahora = ui.input(|i| i.time);
        let inicio = *self.inicio.get_or_insert(ahora);
        let n = ((ahora - inicio).max(0.0) * self.velocidad as f64) as usize;
        if n >= total {
            self.animando = false;
            total
        } else {
            ui.ctx().request_repaint();
            n
        }
    }
}
