mod graph;

use iced::{
    event,
    keyboard::key::Code,
    widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text as CanvasText},
    Element, Length, Pixels, Point, Size, Subscription, Task, Theme, Vector,
};
use iced_widget::{button, text, Column, Container};

use graph::{DescStorage, Node};

pub fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Main,
    OpenAddMenu,
    AddNode(usize),
    CursorMoved(Point),
    MousePressed,
    MouseReleased,
}

impl Message {
    fn from_event(event: iced::Event) -> Message {
        match event {
            iced::Event::Mouse(mouse_event) => match mouse_event {
                iced::mouse::Event::CursorMoved { position } => Message::CursorMoved(position),
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    Message::MousePressed
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                    Message::MouseReleased
                }
                _ => Message::Main,
            },
            iced::Event::Keyboard(key_event) => match key_event {
                iced::keyboard::Event::KeyPressed {
                    physical_key,
                    modifiers,
                    ..
                } => {
                    if physical_key == Code::KeyA && modifiers.shift() {
                        Message::OpenAddMenu
                    } else {
                        Message::Main
                    }
                }
                _ => Message::Main,
            },
            _ => Message::Main,
        }
    }
}

struct Connection {
    from: (usize, usize),
    to: (usize, usize),
}

enum Dragging {
    Node { node_id: usize, offset: Vector },
    Connection { node_id: usize, port_id: usize },
}

struct App {
    nodes: Vec<Node>,
    desc_storage: DescStorage,
    connections: Vec<Connection>,
    dragging: Option<Dragging>,
    cursor: Point,

    add_menu: Option<Point>,
}

impl Default for App {
    fn default() -> Self {
        let yaml = std::fs::read_to_string("nodes.yaml").unwrap_or_default();
        let desc_storage = DescStorage::from(yaml).expect("Failed to load node descriptions");
        Self {
            nodes: vec![],
            desc_storage,
            connections: vec![],
            dragging: None,
            cursor: Point::ORIGIN,

            add_menu: None,
        }
    }
}

impl App {
    pub fn title(&self) -> String {
        "Node Graph".into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::from_event)
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Main => Task::none(),
            Message::OpenAddMenu => {
                self.add_menu = Some(self.cursor);
                Task::none()
            }
            Message::AddNode(index) => {
                let desc = &self.desc_storage.descs[index];
                let new_node = Node {
                    id: self.nodes.len(),
                    pos: self.add_menu.unwrap_or(self.cursor),
                    size: Size::new(
                        120.0,
                        30.0 + ((desc.inputs.len() + desc.outputs.len()) as f32) * 20.0,
                    ),
                    desc: desc.clone(),
                };
                self.nodes.push(new_node);
                self.add_menu = None;
                Task::none()
            }
            Message::CursorMoved(p) => {
                self.cursor = p;
                match &mut self.dragging {
                    Some(Dragging::Node { node_id, offset }) => {
                        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == *node_id) {
                            let new_pos = p - *offset;
                            node.pos = Point::new(new_pos.x.max(0.0), new_pos.y.max(0.0));
                        }
                    }
                    Some(Dragging::Connection { .. }) => {}
                    None => {}
                }
                Task::none()
            }
            Message::MousePressed => {
                for node in &self.nodes {
                    let cursor_pos = self.cursor;
                    if cursor_pos.x >= node.pos.x
                        && cursor_pos.x <= node.pos.x + node.size.width
                        && cursor_pos.y >= node.pos.y
                        && cursor_pos.y <= node.pos.y + 20.0
                    {
                        self.dragging = Some(Dragging::Node {
                            node_id: node.id,
                            offset: self.cursor - node.pos,
                        });
                    }

                    for i in 0..node.desc.outputs.len() {
                        let port_pos = self.port_position(node.id, i, true);
                        let diff = port_pos - self.cursor;
                        let distance = (diff.x.powi(2) + diff.y.powi(2)).sqrt();
                        if distance < 10.0 {
                            self.dragging = Some(Dragging::Connection {
                                node_id: node.id,
                                port_id: i,
                            });
                            return Task::none();
                        }
                    }
                }

                Task::none()
            }
            Message::MouseReleased => {
                match self.dragging.take() {
                    Some(Dragging::Connection { node_id, port_id }) => {
                        for node in &self.nodes {
                            for i in 0..node.desc.inputs.len() {
                                let port_pos = self.port_position(node.id, i, false);
                                let diff = port_pos - self.cursor;
                                let distance = (diff.x.powi(2) + diff.y.powi(2)).sqrt();
                                if distance < 10.0 {
                                    self.connections
                                        .retain(|conn| !(conn.to.0 == node.id && conn.to.1 == i));
                                    self.connections.push(Connection {
                                        from: (node_id, port_id),
                                        to: (node.id, i),
                                    });
                                    self.verify_connections();
                                    return Task::none();
                                }
                            }
                        }
                    }
                    Some(Dragging::Node { .. }) => {}
                    None => {}
                }
                self.dragging = None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let canvas = Canvas::new(NodeGraph { app: self })
            .width(Length::Fill)
            .height(Length::Fill);

        if let Some(_menu_pos) = self.add_menu {
            let mut menu_column = Column::new().spacing(2);
            for (i, desc) in self.desc_storage.descs.iter().enumerate() {
                let btn = button(text(&desc.title)).on_press(Message::AddNode(i));
                menu_column = menu_column.push(btn);
            }

            let menu = Container::new(menu_column)
                .width(Length::Fixed(200.0))
                .padding(5);

            iced::widget::Stack::new()
                .push(canvas)
                .push(menu) // menu will be on top
                .into()
        } else {
            canvas.into()
        }
    }

    fn verify_connections(&mut self) {
        self.connections.retain(|conn| {
            let from_exists = self.nodes.iter().any(|n| n.id == conn.from.0);
            let to_exists = self.nodes.iter().any(|n| n.id == conn.to.0);
            from_exists && to_exists
        });
        self.connections.retain(|conn| conn.from.0 != conn.to.0);
    }

    fn port_position(&self, node_id: usize, port_index: usize, output: bool) -> Point {
        let node = self.nodes.iter().find(|n| n.id == node_id).unwrap();
        let y = node.pos.y + 30.0 + port_index as f32 * 20.0;
        let x = if output {
            node.pos.x + 120.0
        } else {
            node.pos.x
        };
        Point::new(x, y)
    }
}

struct NodeGraph<'a> {
    app: &'a App,
}

impl<'a> NodeGraph<'a> {
    fn draw_connection(&self, frame: &mut Frame, from: Point, to: Point) {
        let path = Path::line(from, to);

        frame.stroke(
            &path,
            Stroke::default()
                .with_color(iced::Color::parse("#6D94C5").unwrap())
                .with_width(2.0),
        );
    }

    fn draw_node(&self, frame: &mut Frame, node: &Node) {
        // Node background
        let rect = Path::rectangle(node.pos, node.size);
        frame.fill(&rect, iced::Color::from_rgb(0.2, 0.2, 0.3));
        frame.stroke(
            &rect,
            Stroke::default().with_color(iced::Color::parse("#19183B").unwrap()),
        );

        // Title bar
        let title_bar = Path::rectangle(node.pos, iced::Size::new(node.size.width, 20.0));
        frame.fill(&title_bar, iced::Color::from_rgb(0.1, 0.1, 0.2));

        // Inputs
        for i in 0..node.desc.inputs.len() {
            let p = self.app.port_position(node.id, i, false);
            let circle = Path::circle(p, 5.0);
            frame.fill(&circle, iced::Color::from_rgb(0.7, 0.2, 0.2));
        }

        // Outputs
        for i in 0..node.desc.outputs.len() {
            let p = self.app.port_position(node.id, i, true);
            let circle = Path::circle(p, 5.0);
            frame.fill(&circle, iced::Color::from_rgb(0.2, 0.7, 0.2));
        }

        // Title text
        let title_pos = Point::new(node.pos.x + 2.0, node.pos.y);
        frame.fill_text(CanvasText {
            content: node.desc.title.clone(),
            position: title_pos,
            color: iced::Color::WHITE,
            size: Pixels(16.0),
            ..Default::default()
        });
    }
}

impl<'a> canvas::Program<Message> for NodeGraph<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _curson: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        for node in &self.app.nodes {
            self.draw_node(&mut frame, node);
        }

        for conn in &self.app.connections {
            self.draw_connection(
                &mut frame,
                self.app.port_position(conn.from.0, conn.from.1, true),
                self.app.port_position(conn.to.0, conn.to.1, false),
            );
        }

        match &self.app.dragging {
            Some(Dragging::Node { node_id, .. }) => {
                if let Some(node) = self.app.nodes.iter().find(|n| n.id == *node_id) {
                    let highlight = Path::rectangle(node.pos, node.size);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(iced::Color::parse("#FFD700").unwrap())
                            .with_width(3.0),
                    );
                }
            }
            Some(Dragging::Connection { node_id, port_id }) => {
                self.draw_connection(
                    &mut frame,
                    self.app.port_position(*node_id, *port_id, true),
                    self.app.cursor,
                );
            }
            _ => {}
        }

        vec![frame.into_geometry()]
    }
}
