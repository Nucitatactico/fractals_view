use eframe::egui;

use crate::fractales::{
    arbol_pitagoras::ArbolPitagoras,
    arbol_pitagoras_paralelo::ArbolPitagorasParalelo,
    copo_koch::CopoKoch,
    copo_koch_paralelo::CopoKochParalelo,
    dragon::Dragon,
    romanesco::Romanesco,
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

fn menu_principal(ui: &mut egui::Ui, seleccion: &mut Option<Box<dyn Fractal>>) {
    let ancho_ventana = ui.available_width();
    let ancho_boton = (ancho_ventana * 0.6).clamp(400.0, 900.0);
    let alto_boton = 56.0;

    let botones: [(&str, fn() -> Box<dyn Fractal>); 6] = [
        (" Árbol de Pitágoras", || Box::new(ArbolPitagoras::default())),
        (" Árbol de Pitágoras (Paralelo)", || Box::new(ArbolPitagorasParalelo::default())),
        (" Copo de Koch", || Box::new(CopoKoch::default())),
        (" Copo de Koch (Paralelo)", || Box::new(CopoKochParalelo::default())),
        (" Curva del Dragón", || Box::new(Dragon::default())),
        (" Brócoli Romanesco", || Box::new(Romanesco::default())),
    ];

    let alto_contenido = botones.len() as f32 * (alto_boton + 12.0) + 120.0;
    let espacio_arriba = ((ui.available_height() - alto_contenido) / 2.0).max(20.0);

    ui.vertical_centered(|ui| {
        ui.add_space(espacio_arriba);
        ui.heading(
            egui::RichText::new("Seleccione la animación")
                .size(32.0)
                .strong(),
        );
        ui.add_space(30.0);

        for (texto, constructor) in botones {
            let boton = egui::Button::new(egui::RichText::new(texto).size(20.0))
                .min_size(egui::vec2(ancho_boton, alto_boton))
                .fill(ui.visuals().widgets.inactive.bg_fill);

            if ui.add(boton).clicked() {
                *seleccion = Some(constructor());
            }
            ui.add_space(12.0);
        }
    });
}

fn vista_fractal(ui: &mut egui::Ui, fractal: &mut Box<dyn Fractal>) -> bool {
    let mut volver = false;

    egui::Panel::top("controles").show_inside(ui, |ui| {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(egui::RichText::new("← Volver").size(18.0))
                        .min_size(egui::vec2(120.0, 40.0)),
                )
                .clicked()
            {
                volver = true;
            }
            ui.separator();
            ui.heading(egui::RichText::new(fractal.nombre()).size(24.0));
        });
        ui.separator();
        fractal.ui_controles(ui);
        ui.add_space(6.0);
    });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        let rect = ui.available_rect_before_wrap();
        fractal.dibujar(ui, rect);
    });

    volver
}

impl eframe::App for MyPixelArt {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match &mut self.fractal {
            None => menu_principal(ui, &mut self.fractal),
            Some(fractal) => {
                let volver = vista_fractal(ui, fractal);
                if volver {
                    self.fractal = None;
                }
            }
        }
    }
}
