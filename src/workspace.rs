use anyhow::{bail, Context as AnyhowContext, Result};
use eframe::egui::{
    Color32, Context, Frame, Id, Label, LayerId, Order, Pos2, Shadow, Stroke, TextEdit,
    TextWrapMode, Ui, Window,
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use tracing::error;

use crate::{
    compiler::Compilation,
    graph::{
        Connection, DescStorage, FieldDesc, FieldKind, Node, PortDesc, PortLocation, PortVariant,
        Type, Var,
    },
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
    pub stabilize_frames: usize,
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
            stabilize_frames: 10,
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
            stabilize_frames: 10,
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
            let from_pos = self.data.nodes[conn.from.0].ports()[conn.from.1].1;
            let to_pos = self.data.nodes[conn.to.0].ports()[conn.to.1].1;
            painter_bg.line_segment([from_pos, to_pos], Stroke::new(2.0, conn.variant.color()));
        }
    }

    fn render_ports(&mut self, ctx: &Context) {
        let painter_fg = ctx.layer_painter(LayerId::new(Order::Background, Id::new("ports_layer")));

        for node in &self.data.nodes {
            for (desc, pos, _location) in node.ports() {
                let size = match desc.variant {
                    PortVariant::Execution => 7.5,
                    PortVariant::Simple => 5.0,
                };

                painter_fg.circle_filled(pos, size, desc.variant.color());
            }
        }
    }

    fn render_nodes(&mut self, ctx: &Context) {
        for node in &mut self.data.nodes {
            let id = Id::new(format!("{}", node.id));
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
                    stroke: Stroke::new(1.0, Color32::from_gray(100)),
                    shadow: Shadow::NONE,
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.set_min_height(node.size.1);
                    ui.set_min_width(node.size.0);
                    if self.stabilize_frames > 0 {
                        self.stabilize_frames -= 1;
                    } else {
                        node.pos.0 = ui.min_rect().min.x - 7.0;
                        node.pos.1 = ui.min_rect().min.y - 38.0;
                    }

                    let short_desc = if node.desc.desc.len() > 18 {
                        format!("{}...", node.desc.desc.chars().take(18).collect::<String>())
                    } else {
                        node.desc.desc.clone()
                    };
                    ui.add(Label::new(short_desc).wrap_mode(TextWrapMode::Extend))
                        .on_hover_text(node.desc.desc.clone());

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

            let ports = &self.data.nodes[from_node].ports();
            let mouse_over_ports = self.mouse_over_ports(self.shared.borrow().cursor);

            let from_pos = ports[from_port].1;
            let color = match mouse_over_ports
                .clone()
                .into_iter()
                .filter(|(_, _, _, location)| *location == PortLocation::Input)
                .count()
            {
                0 => Color32::from_rgb(179, 51, 51),
                1 => Color32::from_rgb(51, 179, 51),
                _ => Color32::from_rgb(179, 179, 51),
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
        let mut to_ports = Vec::new();
        let unique_conns = self
            .data
            .connections
            .iter()
            .rev()
            .filter(|conn| {
                let key = (conn.to.0, conn.to.1);
                if to_ports.contains(&key) {
                    false
                } else {
                    to_ports.push(key);
                    true
                }
            })
            .cloned()
            .collect::<Vec<_>>();
        self.data.connections = unique_conns.into_iter().rev().collect();
    }

    pub fn mouse_over_ports(
        &self,
        mouse_pos: Pos2,
    ) -> Vec<((usize, usize), PortDesc, Pos2, PortLocation)> {
        let mut ports = Vec::new();

        for node in &self.data.nodes {
            for (i, (desc, pos, location)) in node.ports().iter().enumerate() {
                let vertical_dist = (pos.y - mouse_pos.y).abs();
                let horizontal_dist = (pos.x - mouse_pos.x).abs();
                if vertical_dist < 10.0 && horizontal_dist < 30.0 {
                    ports.push(((node.id, i), desc.clone(), *pos, location.clone()));
                }
            }
        }

        ports
    }

    pub fn add_node(&mut self, category: String, title: String) {
        if let Some(desc) = self.data.desc_storage.desc(&category, &title) {
            self.data.nodes.push(Node {
                id: self.data.nodes.len(),
                pos: self.shared.borrow().add_menu.clone().unwrap().0.into(),
                size: (
                    120.0,
                    ((desc.inputs.len() + desc.outputs.len()) as f32 + 1.0) * 20.0,
                ),
                desc: desc.clone(),
            });
            self.shared.borrow_mut().add_menu = None;
        } else {
            error!("Node description not found for {}:{}", category, title);
        }
    }
}
