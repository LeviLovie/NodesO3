use iced::{
    event,
    widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text as CanvasText},
    Element, Pixels, Point, Size, Subscription, Task, Theme, Vector,
};

pub fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Main,
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
            _ => Message::Main,
        }
    }
}

struct Node {
    id: usize,
    title: String,
    pos: Point,
    size: Size,
    inputs: usize,
    outputs: usize,
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
    connections: Vec<Connection>,
    dragging: Option<Dragging>,
    cursor: Point,
}

impl Default for App {
    fn default() -> Self {
        Self {
            nodes: vec![
                Node {
                    id: 0,
                    title: "Const".into(),
                    pos: Point::new(100.0, 100.0),
                    size: Size::new(120.0, 60.0),
                    inputs: 0,
                    outputs: 1,
                },
                Node {
                    id: 1,
                    title: "Const".into(),
                    pos: Point::new(100.0, 200.0),
                    size: Size::new(120.0, 60.0),
                    inputs: 0,
                    outputs: 1,
                },
                Node {
                    id: 2,
                    title: "Add".into(),
                    pos: Point::new(400.0, 200.0),
                    size: Size::new(120.0, 60.0),
                    inputs: 2,
                    outputs: 1,
                },
            ],
            connections: vec![],
            dragging: None,
            cursor: Point::ORIGIN,
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
            Message::CursorMoved(p) => {
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
                self.cursor = p;
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

                    for i in 0..node.outputs {
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
                            for i in 0..node.inputs {
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
        Canvas::new(NodeGraph { app: self })
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
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
        for i in 0..node.inputs {
            let p = self.app.port_position(node.id, i, false);
            let circle = Path::circle(p, 5.0);
            frame.fill(&circle, iced::Color::from_rgb(0.7, 0.2, 0.2));
        }

        // Outputs
        for i in 0..node.outputs {
            let p = self.app.port_position(node.id, i, true);
            let circle = Path::circle(p, 5.0);
            frame.fill(&circle, iced::Color::from_rgb(0.2, 0.7, 0.2));
        }

        // Title text
        let title_pos = Point::new(node.pos.x + 2.0, node.pos.y);
        frame.fill_text(CanvasText {
            content: node.title.clone(),
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
