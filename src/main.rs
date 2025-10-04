mod graph;
mod workspace;

use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, Context as AnyhowContext, Result};
use eframe::egui::{self, CentralPanel, Context, Grid, MenuBar, Pos2, TopBottomPanel, Window};

use graph::Connection;
use workspace::Workspace;

struct Shared {
    pub cursor: Pos2,
    pub add_menu: Option<(Pos2, Option<String>)>,
}

struct App {
    workspace: Workspace,
    shared: Rc<RefCell<Shared>>,
}

impl App {
    fn new() -> Result<Self> {
        let shared = Rc::new(RefCell::new(Shared {
            cursor: Pos2::ZERO,
            add_menu: None,
        }));
        let mut workspace = Workspace::new(shared.clone());
        let yaml = std::fs::read_to_string("nodes.yaml").context("Failed to read nodes.yaml")?;
        workspace.load(yaml).context("Failed to load nodes.yaml")?;

        Ok(Self { workspace, shared })
    }

    fn set_style(ctx: &Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::WHITE);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
        );
        ctx.set_style(style);
    }

    fn render_add_node(&mut self, ctx: &Context) {
        if self.shared.borrow().add_menu.is_none() {
            return;
        }

        let add_menu = self.shared.borrow().add_menu.clone().unwrap();
        let mut reset_menu = false;
        let mut add: Option<usize> = None;

        Window::new(format!(
            "Add {}",
            add_menu.clone().1.unwrap_or("Node".to_string())
        ))
        .fixed_pos(add_menu.0)
        .collapsible(false)
        .title_bar(true)
        .resizable(false)
        .frame(egui::Frame {
            inner_margin: 4.0.into(),
            corner_radius: 0.into(),
            fill: egui::Color32::from_hex("#202020").unwrap(),
            stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
            shadow: egui::epaint::Shadow::NONE,
            ..Default::default()
        })
        .show(ctx, |ui| {
            if let Some(category) = add_menu.clone().1 {
                if ui.button("Back").clicked() {
                    reset_menu = true;
                }

                for (i, desc) in self.workspace.data.desc_storage.descs.iter().enumerate() {
                    if desc.category == *category && ui.button(&desc.title).clicked() {
                        add = Some(i);
                        reset_menu = true;
                    }
                }
            } else {
                for category in &self.workspace.data.desc_storage.categories {
                    if ui.button(category).clicked() {
                        self.shared.borrow_mut().add_menu =
                            Some((add_menu.clone().0, Some(category.clone())));
                    }
                }
            }
        });

        if reset_menu {
            self.shared.borrow_mut().add_menu = Some((add_menu.0, None));
        }

        if let Some(i) = add {
            self.workspace.add_node(i);
            self.shared.borrow_mut().add_menu = None;
        }
    }

    fn render_menu_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    Grid::new("File").show(ui, |ui| {
                        if ui.button("New").clicked() {
                            println!("New file logic...");
                            ui.close();
                        }
                        ui.end_row();

                        if ui.button("Open").clicked() {
                            println!("Open file logic...");
                            ui.close();
                        }
                        ui.label("Shift+O");
                        ui.end_row();

                        if ui.button("Save").clicked() {
                            println!("Save file logic...");
                            ui.close();
                        }
                        ui.label("Shift+S");
                        ui.end_row();
                    });
                });

                ui.menu_button("Nodes", |ui| {
                    Grid::new("Nodes").show(ui, |ui| {
                        if ui.button("Add").clicked() {
                            self.shared.borrow_mut().add_menu =
                                Some((Pos2::new(100.0, 100.0), None));
                            ui.close();
                        }
                        ui.label("Shift+A");
                        ui.end_row();
                    });
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        ui.close();
                    }
                });
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        App::set_style(ctx);

        let input = ctx.input(|i| i.clone());
        if let Some(pos) = input.pointer.hover_pos() {
            self.shared.borrow_mut().cursor = pos;
        }
        if let Some((from_node, from_port, _)) = self.workspace.dragging_connection {
            self.workspace.dragging_connection =
                Some((from_node, from_port, self.shared.borrow().cursor));
        }

        if input.key_pressed(egui::Key::A) && input.modifiers.shift {
            let cursor = self.shared.borrow().cursor;
            self.shared.borrow_mut().add_menu = Some((cursor, None));
        }

        if ctx.input(|i| i.pointer.any_pressed())
            && self.workspace.dragging_connection.is_none()
            && let Some((node_id, port_id, _)) = self
                .workspace
                .mouse_over_port(self.shared.borrow().cursor, true)
        {
            self.workspace.dragging_connection =
                Some((node_id, port_id, self.shared.borrow().cursor));
        }

        if ctx.input(|i| i.pointer.any_released())
            && let Some((from_node_id, from_port_id, current_pos)) =
                self.workspace.dragging_connection.take()
            && let Some((to_node_id, to_port_id, _)) =
                self.workspace.mouse_over_port(current_pos, false)
        {
            self.workspace.data.connections.push(Connection {
                from: (from_node_id, from_port_id),
                to: (to_node_id, to_port_id),
            });
            self.workspace.verify_connections();
        }

        CentralPanel::default().show(ctx, |_| {});

        self.workspace.update(ctx);

        self.render_add_node(ctx);
        self.render_menu_bar(ctx);

        ctx.request_repaint();
    }
}

fn main() -> Result<()> {
    let app = App::new().context("Creating app")?;
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Node Graph",
        options,
        Box::new(|_cc| Ok(Box::new(app) as Box<dyn eframe::App>)),
    )
    .map_err(|e| anyhow!(format!("{e:?}")))
    .context("Running eframe")?;
    Ok(())
}
