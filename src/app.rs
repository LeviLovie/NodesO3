use anyhow::Result;
use eframe::egui::{
    self, Align, CentralPanel, Context, Grid, Layout, MenuBar, Pos2, RichText, TopBottomPanel,
    Window,
};
use std::{cell::RefCell, rc::Rc};
use tracing::{error, info};

use crate::{
    graph::{PortLocation, PortVariant},
    Connection, DialogPurpose, FilePicker, Workspace,
};

pub struct Shared {
    pub cursor: Pos2,
    pub add_menu: Option<(Pos2, Option<String>)>,
    pub error: Option<String>,
    pub compile_debug_info: bool,
}

pub struct App {
    workspace: Option<Workspace>,
    shared: Rc<RefCell<Shared>>,
    picker: Option<FilePicker>,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            workspace: None,
            shared: Rc::new(RefCell::new(Shared {
                cursor: Pos2::ZERO,
                add_menu: None,
                error: None,
                compile_debug_info: false,
            })),
            picker: None,
        })
    }

    fn new_workspace(&mut self) -> Result<()> {
        let workspace = Workspace::new(self.shared.clone());
        self.workspace = Some(workspace);
        Ok(())
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

    // self.workspace.unwrap() is safe since this is only called after the check int he start of the function.
    fn render_add_node(&mut self, ctx: &Context) {
        if self.shared.borrow().add_menu.is_none() {
            return;
        }
        if self.workspace.is_none() {
            self.shared.borrow_mut().error = Some(
                "App::render_add_node has been called with workspace == None... WHY?!".to_string(),
            );
            return;
        }

        let add_menu = self.shared.borrow().add_menu.clone().unwrap();
        let mut reset_menu = false;
        let mut add: Option<(String, String)> = None;

        Window::new(format!(
            "Add {}",
            add_menu.clone().1.unwrap_or("Node".to_string())
        ))
        .fixed_pos(add_menu.0)
        .collapsible(false)
        .title_bar(true)
        .movable(true)
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
                if ui.button("Close").clicked() {
                    self.shared.borrow_mut().add_menu = None;
                }
                if ui.button("Back").clicked() {
                    reset_menu = true;
                }
                ui.add_space(8.0);

                if let Some(descs) = self
                    .workspace
                    .as_ref()
                    .unwrap()
                    .data
                    .desc_storage
                    .descs_category(&category)
                {
                    for desc in descs {
                        if ui.button(&desc.title).clicked() {
                            add = Some((category.clone(), desc.title.clone()));
                            reset_menu = true;
                        }
                    }
                }
            } else {
                if ui.button("Close").clicked() {
                    self.shared.borrow_mut().add_menu = None;
                }
                ui.add_space(8.0);

                for category in &self
                    .workspace
                    .as_ref()
                    .unwrap()
                    .data
                    .desc_storage
                    .categories()
                {
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

        if let Some((category, title)) = add {
            self.workspace.as_mut().unwrap().add_node(category, title);
            self.shared.borrow_mut().add_menu = None;
        }
    }

    fn render_menu_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    Grid::new("File").show(ui, |ui| {
                        if ui.button("New").clicked() {
                            if let Err(e) = self.new_workspace() {
                                self.shared.borrow_mut().error =
                                    Some(format!("Failed to create new workspace: {e:?}"));
                            }
                            ui.close();
                        }
                        ui.end_row();

                        if ui.button("Open").clicked() {
                            self.picker = Some(FilePicker::new(DialogPurpose::OpenWorkspace));
                            ui.close();
                        }
                        ui.label("Shift+O");
                        ui.end_row();

                        if ui.button("Save").clicked() {
                            if self.workspace.is_some() {
                                self.picker = Some(FilePicker::new(DialogPurpose::SaveWorkspace));
                            }

                            ui.close();
                        }
                        ui.label("Shift+S");
                        ui.end_row();
                    });
                });

                ui.menu_button("Nodes", |ui| {
                    Grid::new("Nodes").show(ui, |ui| {
                        if ui.button("Import Libs").clicked() {
                            self.picker = Some(FilePicker::new(DialogPurpose::ImportLibs));
                            ui.close();
                        }
                        ui.end_row();

                        if ui.button("Import Std Libs").clicked() {
                            self.workspace
                                .as_mut()
                                .unwrap()
                                .data
                                .desc_storage
                                .import_std_libs()
                                .unwrap_or_else(|e| {
                                    error!("Failed to import std libs: {e:?}");
                                    self.shared.borrow_mut().error =
                                        Some(format!("Failed to import std libs: {e:?}"))
                                });
                            ui.close();
                        }
                        ui.end_row();

                        if ui.button("Add").clicked() {
                            self.shared.borrow_mut().add_menu =
                                Some((Pos2::new(100.0, 100.0), None));
                            ui.close();
                        }
                        ui.label("Shift+A");
                        ui.end_row();
                    });
                });

                ui.menu_button("Compile", |ui| {
                    ui.checkbox(
                        &mut self.shared.borrow_mut().compile_debug_info,
                        "Include debug info",
                    );

                    // let final_nodes = self
                    //     .workspace
                    //     .as_ref()
                    //     .unwrap()
                    //     .data
                    //     .nodes
                    //     .iter()
                    //     .filter(|n| n.desc.end)
                    //     .collect::<Vec<_>>();
                    // if ui
                    //     .add_enabled(final_nodes.len() == 1, Button::new("Compile"))
                    //     .clicked()
                    // {
                    //     let mut compiler = Compiler::new(
                    //         self.shared.borrow().compile_debug_info,
                    //         self.workspace.as_ref().unwrap().data.nodes.clone(),
                    //         self.workspace.as_ref().unwrap().data.connections.clone(),
                    //         final_nodes[0].id,
                    //     );
                    //     match compiler.compile() {
                    //         Ok(compilation) => {
                    //             self.workspace.as_mut().unwrap().data.compilation =
                    //                 Some(compilation);
                    //         }
                    //         Err(e) => {
                    //             error!("Compilation failed: {e:?}");
                    //             self.shared.borrow_mut().error =
                    //                 Some(format!("Compilation failed: {e:?}"));
                    //         }
                    //     }
                    //     ui.close();
                    // }
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

        if let Some(picker) = &mut self.picker {
            picker.show(ctx);
            if let Some(path) = &picker.picked_path {
                match picker.purpose {
                    DialogPurpose::OpenWorkspace => {
                        info!(?path, "Opening workspace");
                        let compressed = path
                            .extension()
                            .map(|ext| ext == "no3zstd")
                            .unwrap_or(false);
                        match Workspace::load(self.shared.clone(), path.clone(), compressed) {
                            Ok(ws) => {
                                self.workspace = Some(ws);
                            }
                            Err(e) => {
                                error!("Failed to open workspace: {e:?}");
                                self.shared.borrow_mut().error =
                                    Some(format!("Failed to open workspace: {e:?}"))
                            }
                        }
                    }
                    DialogPurpose::SaveWorkspace => {
                        info!(?path, "Saving workspace");
                        let compress = path
                            .extension()
                            .map(|ext| ext == "no3zstd")
                            .unwrap_or(false);
                        if self.workspace.is_none() {
                            error!("No workspace to save.");
                            self.shared.borrow_mut().error =
                                Some("No workspace to save.".to_string());
                        } else if let Err(e) = self
                            .workspace
                            .as_ref()
                            .unwrap()
                            .save(path.clone(), compress)
                        {
                            error!("Failed to save workspace: {e:?}");
                            self.shared.borrow_mut().error =
                                Some(format!("Failed to save workspace: {e:?}"));
                        }
                    }
                    DialogPurpose::ImportLibs => {
                        let paths = &picker.picked_paths;
                        info!(
                            count = paths.as_ref().map(|p| p.len()).unwrap_or(0),
                            "Importing libs"
                        );
                        for path in paths.as_ref().unwrap_or(&vec![]) {
                            info!(?path, "Importing lib");
                            if self.workspace.is_none() {
                                self.shared.borrow_mut().error =
                                    Some("No workspace to import lib into.".to_string());
                            } else if let Err(e) = self
                                .workspace
                                .as_mut()
                                .unwrap()
                                .data
                                .desc_storage
                                .load_import(path.to_path_buf(), true)
                            {
                                self.shared.borrow_mut().error =
                                    Some(format!("Failed to import lib: {e:?}"));
                            }
                        }
                    }
                    _ => {}
                }
                self.picker = None;
            }
        }

        if self.workspace.is_some() {
            if let Some((from_node, from_port, _)) =
                self.workspace.as_ref().unwrap().dragging_connection
            {
                self.workspace.as_mut().unwrap().dragging_connection =
                    Some((from_node, from_port, self.shared.borrow().cursor));
            }

            if input.key_pressed(egui::Key::A) && input.modifiers.shift {
                let cursor = self.shared.borrow().cursor;
                self.shared.borrow_mut().add_menu = Some((cursor, None));
            }

            let ports = self
                .workspace
                .as_mut()
                .unwrap()
                .mouse_over_ports(self.shared.borrow().cursor);

            if ctx.input(|i| i.pointer.any_pressed())
                && self
                    .workspace
                    .as_ref()
                    .unwrap()
                    .dragging_connection
                    .is_none()
            {
                for ((node_id, port_id), _, _, location) in &ports {
                    match location {
                        PortLocation::Output => {
                            self.workspace.as_mut().unwrap().dragging_connection =
                                Some((*node_id, *port_id, self.shared.borrow().cursor));
                        }
                        PortLocation::Input => {
                            if self
                                .workspace
                                .as_mut()
                                .unwrap()
                                .data
                                .connections
                                .iter()
                                .any(|c| c.to.0 == *node_id && c.to.1 == *port_id)
                                && let Some((from_node, from_port)) = self
                                    .workspace
                                    .as_ref()
                                    .unwrap()
                                    .data
                                    .connections
                                    .iter()
                                    .find(|c| c.to.0 == *node_id && c.to.1 == *port_id)
                                    .map(|c| c.from)
                            {
                                self.workspace
                                    .as_mut()
                                    .unwrap()
                                    .data
                                    .connections
                                    .retain(|c| !(c.to.0 == *node_id && c.to.1 == *port_id));
                                self.workspace.as_mut().unwrap().dragging_connection =
                                    Some((from_node, from_port, self.shared.borrow().cursor));
                            }
                        }
                    }
                }
            }

            if ctx.input(|i| i.pointer.any_released()) {
                for ((node_id, port_id), to_desc, _, location) in &ports {
                    if location == &PortLocation::Input
                        && let Some((from_node_id, from_port_id, _)) =
                            self.workspace.as_mut().unwrap().dragging_connection.take()
                    {
                        let from_node = self
                            .workspace
                            .as_ref()
                            .unwrap()
                            .data
                            .nodes
                            .iter()
                            .find(|n| n.id == from_node_id)
                            .unwrap();
                        let from_ports = from_node.ports();
                        let from_port = from_ports
                            .iter()
                            .enumerate()
                            .find(|(i, _)| *i == from_port_id);
                        if let Some((_, (from_desc, _, _))) = from_port
                            && from_desc.variant == to_desc.variant
                        {
                            self.workspace
                                .as_mut()
                                .unwrap()
                                .data
                                .connections
                                .push(Connection {
                                    variant: PortVariant::Simple,
                                    from: (from_node_id, from_port_id),
                                    to: (*node_id, *port_id),
                                });
                            self.workspace.as_mut().unwrap().verify_connections();
                        }
                    }
                }

                self.workspace.as_mut().unwrap().dragging_connection = None;
            }

            CentralPanel::default().show(ctx, |_| {});

            self.workspace.as_mut().unwrap().update(ctx);

            self.render_add_node(ctx);
        } else {
            CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    let screen_height = ctx.available_rect().height();
                    ui.add_space(screen_height / 4.0);
                    ui.label(RichText::new("No workspace loaded.").size(40.0));
                    ui.add_space(screen_height / 4.0);
                    if ui.button("New Workspace").clicked()
                        && let Err(e) = self.new_workspace()
                    {
                        self.shared.borrow_mut().error =
                            Some(format!("Failed to create new workspace: {e:?}"));
                    }
                    if ui.button("Open Workspace").clicked() {
                        self.picker = Some(FilePicker::new(DialogPurpose::OpenWorkspace));
                    }
                });
            });
        }

        self.render_menu_bar(ctx);

        ctx.request_repaint();
    }
}
