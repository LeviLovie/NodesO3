use anyhow::{Context as AnyhowContext, Result};
use eframe::egui::{Color32, Context, Frame, Id, LayerId, Order, Pos2, Shadow, Stroke, Ui, Window};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

use crate::{
    graph::{Connection, DescStorage, FieldDesc, FieldKind, Node, Var},
    Shared,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkspaceData {
    pub nodes: Vec<Node>,
    pub desc_storage: DescStorage,
    pub connections: Vec<Connection>,
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
                nodes: Vec::new(),
                desc_storage: DescStorage::new(),
                connections: Vec::new(),
            },
            shared,
            dragging_connection: None,
        }
    }

    pub fn load(&mut self, yaml: String) -> Result<()> {
        self.data
            .desc_storage
            .load(yaml)
            .context("Failed to load node descriptions")?;
        Ok(())
    }

    pub fn update(&mut self, ctx: &Context) {
        self.render_connections(ctx);
        self.render_nodes(ctx);
        self.render_ports(ctx);
        self.render_dragging_connection(ctx);
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
        let painter_fg = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("ports_layer")));

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
            Window::new(&node.desc.title)
                .default_pos(node.pos)
                .max_width(node.size.1)
                .resizable(false)
                .collapsible(false)
                .movable(self.dragging_connection.is_none())
                .frame(Frame {
                    inner_margin: 2.0.into(),
                    corner_radius: 0.into(),
                    fill: Color32::from_hex("#202020").unwrap(),
                    stroke: Stroke::new(1.0, Color32::from_gray(100)),
                    shadow: Shadow::NONE,
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.set_min_height(node.size.1);
                    ui.set_min_width(node.size.0);
                    node.pos.0 = ui.min_rect().min.x;
                    node.pos.1 = ui.min_rect().min.y;

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
            FieldKind::Enter => match (&mut field.value, &mut field.raw_value) {
                (Var::Bool(b), _) => {
                    if ui.checkbox(b, "").clicked() {
                        *b = !*b;
                    }
                }
                (Var::String(s), _) => {
                    ui.text_edit_singleline(s);
                }
                (Var::Int(i), raw) => {
                    if ui.text_edit_singleline(raw).lost_focus()
                        && let Ok(new_i) = raw.parse::<i64>()
                    {
                        *i = new_i;
                    }
                }
                (Var::Float(f), raw) => {
                    if ui.text_edit_singleline(raw).lost_focus()
                        && let Ok(new_f) = raw.parse::<f64>()
                    {
                        *f = new_f;
                    }
                }
                (Var::Custom((_, value)), raw) => {
                    if ui.text_edit_singleline(raw).lost_focus() {
                        *value = raw.to_string();
                    }
                }
            },
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

    pub fn add_node(&mut self, index: usize) {
        let desc = &self.data.desc_storage.descs[index];
        self.data.nodes.push(Node {
            id: self.data.nodes.len(),
            pos: self.shared.borrow().add_menu.clone().unwrap().0.into(),
            size: (
                120.0,
                ((desc.inputs.len() + desc.outputs.len()) as f32) * 20.0,
            ),
            desc: desc.clone(),
        });
        self.shared.borrow_mut().add_menu = None;
    }
}
