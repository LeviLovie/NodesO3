mod graph;

use anyhow::{anyhow, Context as AnyhowContext, Result};
use eframe::egui::{self, CentralPanel, Color32, Context, Pos2, Stroke, Window};

use graph::{DescStorage, FieldDesc, FieldKind, Node, Var};

struct Connection {
    from: (usize, usize),
    to: (usize, usize),
}

struct App {
    nodes: Vec<Node>,
    desc_storage: DescStorage,
    connections: Vec<Connection>,
    cursor: Pos2,
    add_menu: Option<Pos2>,
    add_menu_category: Option<String>,
    dragging_connection: Option<(usize, usize, Pos2)>, // (node_id, port_index, current_pos)>,
}

impl App {
    fn new() -> Result<Self> {
        let yaml = std::fs::read_to_string("nodes.yaml").context("Failed to read nodes.yaml")?;
        let desc_storage = DescStorage::from(yaml).context("Failed to load node descriptions")?;
        Ok(Self {
            nodes: vec![],
            desc_storage,
            connections: vec![],
            cursor: Pos2::ZERO,
            add_menu: None,
            add_menu_category: None,
            dragging_connection: None,
        })
    }

    fn port_position(&self, node_id: usize, port_index: usize, output: bool) -> Pos2 {
        let node = self.nodes.iter().find(|n| n.id == node_id).unwrap();
        let y = node.pos.y + 30.0 + port_index as f32 * 20.0;
        let x = if output {
            node.pos.x + node.size.x - 6.0
        } else {
            node.pos.x - 2.0
        };
        Pos2 { x, y }
    }

    fn verify_connections(&mut self) {
        self.connections.retain(|conn| {
            let from_exists = self.nodes.iter().any(|n| n.id == conn.from.0);
            let to_exists = self.nodes.iter().any(|n| n.id == conn.to.0);
            from_exists && to_exists && conn.from.0 != conn.to.0
        });
    }

    fn add_node(&mut self, index: usize) {
        let desc = &self.desc_storage.descs[index];
        let new_node = Node {
            id: self.nodes.len(),
            pos: self.add_menu.unwrap_or(self.cursor),
            size: egui::vec2(
                120.0,
                30.0 + ((desc.inputs.len() + desc.outputs.len()) as f32) * 20.0,
            ),
            desc: desc.clone(),
        };
        self.nodes.push(new_node);
        self.add_menu = None;
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

    fn render_connections(&self, ctx: &Context) {
        let painter_bg = ctx.layer_painter(egui::LayerId::background());

        for conn in &self.connections {
            let from = self.port_position(conn.from.0, conn.from.1, true);
            let to = self.port_position(conn.to.0, conn.to.1, false);
            painter_bg.line_segment(
                [from, to],
                Stroke::new(2.0, Color32::from_rgb(109, 148, 197)),
            );
        }
    }

    fn render_ports(&mut self, ctx: &Context) {
        let painter_fg = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("ports_layer"),
        ));
        let input = ctx.input(|i| i.clone());

        for node in &self.nodes {
            for i in 0..node.desc.inputs.len() {
                let pos = self.port_position(node.id, i, false);
                painter_fg.circle_filled(pos, 5.0, Color32::from_rgb(179, 51, 51));
            }
            for i in 0..node.desc.outputs.len() {
                let pos = self.port_position(node.id, i, true);
                painter_fg.circle_filled(pos, 5.0, Color32::from_rgb(51, 179, 51));

                if ctx.input(|i| i.pointer.any_pressed())
                    && input.pointer.any_pressed()
                    && let Some(press_pos) = input.pointer.press_origin()
                    && press_pos.distance(pos) < 8.0
                {
                    self.dragging_connection = Some((node.id, i, press_pos));
                }
            }
        }
    }

    fn field_edit(ui: &mut egui::Ui, field: &mut FieldDesc) {
        match field.kind {
            FieldKind::Enter => {
                match (&mut field.value, &mut field.raw_value) {
                    (Var::Bool(b), _) => {
                        if ui.checkbox(b, "").clicked() {
                            *b = !*b;
                        }
                    }
                    // Var::String(s) => {
                    //     if ui.text_edit_singleline(&mut text).lost_focus() {
                    //         *s = text;
                    //     }
                    // }
                    (Var::Int(i), raw) => {
                        if ui.text_edit_singleline(raw).lost_focus()
                            && let Ok(new_i) = raw.parse::<i64>()
                        {
                            *i = new_i;
                        }
                    }
                    // Var::Float(f) => {
                    //     if ui.text_edit_singleline(&mut text).lost_focus()
                    //         && let Ok(new_f) = text.parse::<f64>()
                    //     {
                    //         *f = new_f;
                    //     }
                    // }
                    // Var::Custom((_, value)) => {
                    //     if ui.text_edit_singleline(&mut text).lost_focus() {
                    //         *value = text;
                    //     }
                    // }
                    _ => {}
                }
            }
        }
    }

    fn render_nodes(&mut self, ctx: &Context) {
        for node in &mut self.nodes {
            Window::new(&node.desc.title)
                .default_pos(node.pos)
                .max_width(node.size.x)
                .resizable(false)
                .collapsible(false)
                .movable(self.dragging_connection.is_none())
                .frame(egui::Frame {
                    inner_margin: 2.0.into(),
                    corner_radius: 0.into(),
                    fill: egui::Color32::from_hex("#202020").unwrap(),
                    stroke: egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
                    shadow: egui::epaint::Shadow::NONE,
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.set_min_height(node.size.y);
                    ui.set_min_width(node.size.x);
                    node.pos = ui.min_rect().min;

                    for field in &mut node.desc.fields {
                        Self::field_edit(ui, field);
                    }
                });
        }
    }

    fn render_dragging_connection(&mut self, ctx: &Context) {
        if let Some((from_node, from_port, current_pos)) = self.dragging_connection {
            let painter_fg = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("dragging_connection_layer"),
            ));
            let from_pos = self.port_position(from_node, from_port, true);
            painter_fg.line_segment(
                [from_pos, current_pos],
                Stroke::new(2.0, Color32::from_rgb(109, 148, 197)),
            );
            if ctx.input(|i| i.pointer.any_released())
                && let Some(release_pos) = ctx.input(|i| i.pointer.hover_pos())
            {
                for node in &self.nodes {
                    for (i, _) in node.desc.inputs.iter().enumerate() {
                        let port_pos = self.port_position(node.id, i, false);
                        if port_pos.distance(release_pos) < 8.0 {
                            self.connections.push(Connection {
                                from: (from_node, from_port),
                                to: (node.id, i),
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    fn render_add_node(&mut self, ctx: &Context) {
        if self.add_menu.is_none() {
            return;
        }

        let mut reset_menu = false;
        let mut add: Option<usize> = None;

        Window::new(format!(
            "Add {}",
            self.add_menu_category.clone().unwrap_or("Node".to_string())
        ))
        .fixed_pos(self.add_menu.unwrap())
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
            if let Some(category) = &self.add_menu_category {
                if ui.button("Back").clicked() {
                    reset_menu = true;
                }

                for (i, desc) in self.desc_storage.descs.iter().enumerate() {
                    if desc.category == *category && ui.button(&desc.title).clicked() {
                        add = Some(i);
                        reset_menu = true;
                    }
                }
            } else {
                for category in &self.desc_storage.categories {
                    if ui.button(category).clicked() {
                        self.add_menu_category = Some(category.clone());
                    }
                }
            }
        });

        if reset_menu {
            self.add_menu = None;
            self.add_menu_category = None;
        }

        if let Some(i) = add {
            self.add_node(i);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        App::set_style(ctx);

        let input = ctx.input(|i| i.clone());
        if let Some(pos) = input.pointer.hover_pos() {
            self.cursor = pos;
        }
        if input.key_pressed(egui::Key::A) && input.modifiers.shift {
            self.add_menu = Some(self.cursor);
        }
        if let Some((from_node, from_port, _)) = self.dragging_connection {
            self.dragging_connection = Some((from_node, from_port, self.cursor));
        }
        if ctx.input(|i| i.pointer.any_released())
            && let Some((from_node, output_index, current_pos)) = self.dragging_connection.take()
        {
            for node in &self.nodes {
                for i in 0..node.desc.inputs.len() {
                    let port_pos = self.port_position(node.id, i, false);
                    if current_pos.distance(port_pos) < 20.0 {
                        self.connections.push(Connection {
                            from: (from_node, output_index),
                            to: (node.id, i),
                        });
                    }
                }
            }
            self.verify_connections();
        }

        CentralPanel::default().show(ctx, |_| {});

        self.render_connections(ctx);
        self.render_nodes(ctx);
        self.render_ports(ctx);
        self.render_dragging_connection(ctx);
        self.render_add_node(ctx);

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
