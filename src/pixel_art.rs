use eframe::egui;

use crate::fractales::{
    arbol_pitagoras::ArbolPitagoras,
    copo_koch::CopoKoch,
    romanesco::Romanesco,
    apolonio::Apolonio,
    dragon::Dragon,
    Fractal,
};

pub struct MyPixelArt {
    fractal: Option<Box<dyn Fractal>>,
}

impl Default for MyPixelArt {
    fn default() -> Self {
        Self { fractal: None }
    }
}

impl MyPixelArt {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for MyPixelArt {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match &mut self.fractal {
            None => {
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.heading("Seleccione la animación");
                        ui.add_space(30.0);

                        if ui.button("🌳 Árbol de Pitágoras").clicked() {
                            self.fractal = Some(Box::new(ArbolPitagoras::default()));
                        }
                        ui.add_space(10.0);
                        if ui.button("❄️ Copo de Koch").clicked() {
                            self.fractal = Some(Box::new(CopoKoch::default()));
                        }
                        ui.add_space(10.0);
                        if ui.button("🥦 Brócoli Romanesco").clicked() {
                            self.fractal = Some(Box::new(Romanesco::default()));
                        }
                        ui.add_space(10.0);
                        if ui.button("⭕ Tamiz de Apolonio").clicked() {
                            self.fractal = Some(Box::new(Apolonio::default()));
                        }
                        ui.add_space(10.0);
                        if ui.button("🐉 Curva del Dragón").clicked() {
                            self.fractal = Some(Box::new(Dragon::default()));
                        }
                    });
                });
            }
            Some(fractal) => {
                let mut volver = false;
                egui::Panel::top("controles").show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("← Volver").clicked() {
                            volver = true;
                        }
                        ui.separator();
                        ui.heading(fractal.nombre());
                    });
                    ui.separator();
                    fractal.ui_controles(ui);
                });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let rect = ui.available_rect_before_wrap();
                    fractal.dibujar(ui, rect);
                });
                if volver {
                    self.fractal = None;
                }
            }
        }
    }
}
