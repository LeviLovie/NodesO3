use anyhow::{bail, Context as AnyhowContext, Result};
use eframe::egui::{
    Color32, Context, Frame, Id, LayerId, Order, Pos2, Shadow, Stroke, TextEdit, Ui, Window,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use tracing::error;

use crate::{
    compiler::Compilation,
    graph::{Connection, DescStorage, FieldDesc, FieldKind, Node, Type, Var},
    Shared,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkspaceData {
    pub version: String,
    pub nodes: Vec<Node>,
    pub desc_storage: DescStorage,
    pub connections: Vec<Connection>,
    pub compilation: Option<Compilation>,
}

pub struct Workspace {
    pub data: WorkspaceData,
    pub shared: Rc<RefCell<Shared>>,
    pub dragging_connection: Option<(usize, usize, Pos2)>, // (node_id, port_index, current_pos)>,
}

impl Workspace {
    pub fn new(shared: Rc<RefCell<Shared>>) -> Self {
        Self {
            data: WorkspaceData {
                version: env!("CARGO_PKG_VERSION").to_string(),
                nodes: Vec::new(),
                desc_storage: DescStorage::new(),
                connections: Vec::new(),
                compilation: None,
            },
            shared,
            dragging_connection: None,
        }
    }

    #[tracing::instrument(skip(shared))]
    pub fn load(shared: Rc<RefCell<Shared>>, path: PathBuf, compressed: bool) -> Result<Self> {
        let mut ron = std::fs::read(path).context("Failed to read workspace.")?;
        if compressed {
            ron = Self::decompress(&ron)
                .context("Failed to decompress workspace")?
                .as_bytes()
                .to_vec();
        }
        let string_ron = String::from_utf8(ron).context("Failed to convert workspace to string")?;
        let mut data: WorkspaceData =
            ron::from_str(&string_ron).context("Failed to deserialize workspace")?;

        if data.version != env!("CARGO_PKG_VERSION") {
            error!(
                "Workspace version ({}) does not match application version ({}).",
                data.version,
                env!("CARGO_PKG_VERSION")
            );
            bail!(
                "Workspace version ({}) does not match application version ({}).",
                data.version,
                env!("CARGO_PKG_VERSION")
            );
        }

        for node in &mut data.nodes {
            for field in &mut node.desc.fields {
                field.raw_value = field.value.to_string();
            }
        }

        Ok(Self {
            data,
            shared,
            dragging_connection: None,
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn save(&self, path: PathBuf, compress: bool) -> Result<()> {
        let ron = ron::to_string(&self.data).context("Failed to serialize workspace")?;
        if compress {
            let compressed_data = Self::compress(ron).context("Failed to compress workspace")?;
            std::fs::write(path, compressed_data).context("Failed to write workspace.ron")?;
        } else {
            std::fs::write(path, ron).context("Failed to write workspace.ron")?;
        }
        Ok(())
    }

    fn compress(file_data: String) -> Result<Vec<u8>> {
        zstd::encode_all(file_data.as_bytes(), 0).context("Failed to compress data using zstd")
    }

    fn decompress(compressed_data: &[u8]) -> Result<String> {
        String::from_utf8(
            zstd::decode_all(compressed_data).context("Failed to decompress data using zstd")?,
        )
        .context("Failed to convert decompressed data to string")
    }

    pub fn update(&mut self, ctx: &Context) {
        self.render_connections(ctx);
        self.render_ports(ctx);
        self.render_nodes(ctx);
        self.render_dragging_connection(ctx);
        self.render_compilation(ctx);
    }

    fn render_compilation(&mut self, ctx: &Context) {
        if let Some(compilation) = &mut self.data.compilation {
            compilation.update(ctx);
        }
    }

    fn render_connections(&self, ctx: &Context) {
        let painter_bg = ctx.layer_painter(LayerId::background());

        for conn in &self.data.connections {
            let from = self.data.nodes[conn.from.0].port_pos(conn.from.1, true);
            let to = self.data.nodes[conn.to.0].port_pos(conn.to.1, false);
            painter_bg.line_segment(
                [from, to],
                Stroke::new(2.0, Color32::from_rgb(109, 148, 197)),
            );
        }
    }

    fn render_ports(&mut self, ctx: &Context) {
        let painter_fg = ctx.layer_painter(LayerId::new(Order::Background, Id::new("ports_layer")));

        for node in &self.data.nodes {
            for i in 0..node.desc.inputs.len() {
                let pos = node.port_pos(i, false);
                painter_fg.circle_filled(pos, 5.0, Color32::from_rgb(179, 51, 51));
            }
            for i in 0..node.desc.outputs.len() {
                let pos = node.port_pos(i, true);
                painter_fg.circle_filled(pos, 5.0, Color32::from_rgb(51, 179, 51));
            }
        }
    }

    fn render_nodes(&mut self, ctx: &Context) {
        for node in &mut self.data.nodes {
            let id = Id::new(format!("{}", node.id));
            let stroke = if node.desc.end {
                Stroke::new(1.0, Color32::from_hex("#C0C000").unwrap())
            } else {
                Stroke::new(1.0, Color32::from_gray(100))
            };
            Window::new(format!("{}#{}", node.desc.title, node.id))
                .id(id)
                .fixed_pos(node.pos)
                .max_width(node.size.1)
                .resizable(false)
                .collapsible(false)
                .movable(self.dragging_connection.is_none())
                .frame(Frame {
                    inner_margin: 6.0.into(),
                    corner_radius: 0.into(),
                    fill: Color32::from_hex("#202020").unwrap(),
                    stroke,
                    shadow: Shadow::NONE,
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.set_min_height(node.size.1);
                    ui.set_min_width(node.size.0);
                    if node.stabilize_frames < 10 {
                        node.stabilize_frames += 1;
                    } else {
                        node.pos.0 = ui.min_rect().min.x - 7.0;
                        node.pos.1 = ui.min_rect().min.y - 38.0;
                    }

                    for field in &mut node.desc.fields {
                        Self::field_edit(ui, field);
                    }
                });
        }
    }

    fn render_dragging_connection(&mut self, ctx: &Context) {
        if let Some((from_node, from_port, current_pos)) = self.dragging_connection {
            let painter_fg = ctx.layer_painter(LayerId::new(
                Order::Foreground,
                Id::new("dragging_connection_layer"),
            ));
            let from_pos = self.data.nodes[from_node].port_pos(from_port, true);
            let color = if self
                .mouse_over_port(self.shared.borrow().cursor, false)
                .is_some()
            {
                Color32::from_rgb(51, 179, 51)
            } else {
                Color32::from_rgb(179, 51, 51)
            };

            painter_fg.line_segment([from_pos, current_pos], Stroke::new(2.0, color));
        }
    }

    fn field_edit(ui: &mut Ui, field: &mut FieldDesc) {
        match field.kind {
            FieldKind::Enter => {
                if matches!(field.data_type, Type::Bool) {
                    if ui
                        .checkbox(&mut field.value.clone().try_into().unwrap(), "")
                        .on_hover_text(&field.name)
                        .changed()
                    {
                        let current_value: bool = field.value.clone().try_into().unwrap();
                        field.value = (!current_value).into();
                    }
                    return;
                }

                let response = ui.add(
                    TextEdit::singleline(&mut field.raw_value)
                        .hint_text(format!("{}: {}", field.name, field.data_type)),
                );
                if response.lost_focus() {
                    if field.raw_value.is_empty() {
                        field.value = Var::Int(0);
                        return;
                    }

                    let multi = match &field.data_type {
                        Type::Multi(m) => Some(m),
                        _ => None,
                    };

                    #[allow(clippy::collapsible_if)]
                    if matches!(field.data_type, Type::Float)
                        || multi.is_some_and(|m| m.contains(&Type::Float))
                    {
                        if let Ok(v) = field.raw_value.parse::<f64>() {
                            field.value = Var::Float(v);
                            field.raw_value = v.to_string();
                            return;
                        }
                    };

                    #[allow(clippy::collapsible_if)]
                    if matches!(field.data_type, Type::Int)
                        || multi.is_some_and(|m| m.contains(&Type::Int))
                    {
                        if let Ok(v) = field.raw_value.parse::<i64>() {
                            field.value = Var::Int(v);
                            field.raw_value = v.to_string();
                            return;
                        }
                    };

                    #[allow(clippy::collapsible_if)]
                    if multi.is_some_and(|m| m.contains(&Type::Bool)) {
                        if let Ok(v) = field.raw_value.parse::<bool>() {
                            field.value = Var::Bool(v);
                            field.raw_value = v.to_string();
                            return;
                        }
                    };

                    if matches!(field.data_type, Type::String)
                        || multi.is_some_and(|m| m.contains(&Type::String))
                    {
                        field.value = Var::String(field.raw_value.clone());
                        return;
                    };

                    println!(
                        "Failed to parse value for field '{}' ('{}': {})",
                        field.name, field.raw_value, field.data_type
                    );
                }
            }
        }
    }

    pub fn verify_connections(&mut self) {
        self.data
            .connections
            .retain(|conn| conn.from.0 != conn.to.0);
        self.data.connections.retain(|conn| {
            let from_exists = self.data.nodes.iter().any(|n| n.id == conn.from.0);
            let to_exists = self.data.nodes.iter().any(|n| n.id == conn.to.0);
            from_exists && to_exists
        });
    }

    pub fn mouse_over_port(
        &self,
        mouse_pos: Pos2,
        is_output: bool,
    ) -> Option<(usize, usize, bool)> {
        for node in &self.data.nodes {
            if is_output {
                for i in 0..node.desc.outputs.len() {
                    let port_pos = node.port_pos(i, true);
                    let vertical_dist = (port_pos.y - mouse_pos.y).abs();
                    let horizontal_dist = (port_pos.x - mouse_pos.x).abs();
                    if vertical_dist < 10.0 && horizontal_dist < 30.0 {
                        return Some((node.id, i, true));
                    }
                }
            } else {
                for i in 0..node.desc.inputs.len() {
                    let port_pos = node.port_pos(i, false);
                    let vertical_dist = (port_pos.y - mouse_pos.y).abs();
                    let horizontal_dist = (port_pos.x - mouse_pos.x).abs();
                    if vertical_dist < 10.0 && horizontal_dist < 30.0 {
                        return Some((node.id, i, false));
                    }
                }
            }
        }
        None
    }

    pub fn add_node(&mut self, category: String, title: String) {
        if let Some(desc) = self.data.desc_storage.desc(&category, &title) {
            self.data.nodes.push(Node {
                id: self.data.nodes.len(),
                pos: self.shared.borrow().add_menu.clone().unwrap().0.into(),
                size: (
                    120.0,
                    ((desc.inputs.len() + desc.outputs.len()) as f32) * 20.0,
                ),
                desc: desc.clone(),
                stabilize_frames: 0,
            });
            self.shared.borrow_mut().add_menu = None;
        } else {
            error!("Node description not found for {}:{}", category, title);
        }
    }
}
