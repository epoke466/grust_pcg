mod default_usr_settings;
mod usr_settings;
mod view;

use lucide_icons::LUCIDE_FONT_BYTES;

pub use usr_settings::settings::{Config, settings_menu};

use iced::{
    Color, Point, Rectangle, Renderer, Subscription, Task, Theme, Vector, event, keyboard,
    mouse::{self, ScrollDelta},
    widget::{
        canvas::{self, LineDash, Path, Stroke},
        operation::focus,
        row,
    },
};

use shared::{
    DataType::{self, PositionRange, RotationRange, ScaleRange},
    PCGGraph, PCGNode, PCGNodeType, Pin, PinValue, ValueInput, connect_pins, delete_graph_file,
    index_and_pin, load_graph_file, node_from_id, pin_from_uuid, pin_from_uuid_immut,
    save_graph_file, tf, tp, untp,
};

use uuid::Uuid;

use rfd::FileDialog;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::usr_settings::settings::AppTheme;

const SCROLL_SENSITIVIY: f32 = 0.25;
const ZOOM_SENSITIVITY: f32 = 0.002;
const DOT_DENSITY: f32 = 20.0;
const DOT_SIZE: f32 = 1.2;
const TOP_BAR_BUTTON_PADDING: f32 = 6.0;

pub const TITLE_SIZE: f32 = 20.0;
pub const TEXT_SIZE: f32 = 14.0;

#[derive(Debug)]
struct NodePicker {
    visible: bool,
    position: Point,
    search_bar_text: String,
}

#[derive(Debug)]
struct PinInputEditor {
    visible: bool,
    position: Point,
    current_pin_uuid: Uuid,
    data: Option<ValueInput>,
}

#[derive(Debug)]
struct GraphEditor {
    //Structural
    canvas: GraphCanvas,
    node_picker: NodePicker,
    pin_input_editor: PinInputEditor,
    scene: Screen,

    //Graph
    graph_name: String,
    working_directory: PathBuf,
    graph_files: Vec<String>,
    graph: PCGGraph,

    //Mouse
    mouse_position: Point,
    mouse_node_offset: Vector,

    //Draging
    is_draging_node: bool,
    dragging_node: Uuid,

    //Keybinds
    modf: bool,
    deleteing: bool,
    show_keybinds: bool,

    //TextInput
    new_graph_name: String,

    //Settings
    config: Config,
}

#[derive(Debug, Clone)]
pub enum Screen {
    Main,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeTheme(AppTheme),
    OpenDirectoryPicker,
    CloseNodePicker,
    SetMousePosition(iced::Point),
    NodePickerSearch(String),
    SelectNodeType(PCGNodeType),
    LeftClickCanvas,
    RightClickCanvas,
    Scroll(ScrollDelta),
    LoadGraph(String),
    ModifierChanged(bool),
    DeleteingChanged(bool),
    AddNewGraph,
    NewGraphNameUpdate(String),
    SaveGraph,
    SetPinData(Uuid, String, usize, String),
    ClosePinInputEditor,
    DeleteGraph(String),
    ChangeScreen(Screen),
    ToggleKeybinds,
}

impl GraphEditor {
    //Ik this func is incomprehensable but thats the nature of accesign fs in rust
    fn update_graph_list(&mut self) {
        self.graph_files.clear();
        for entry in WalkDir::new(&self.working_directory).max_depth(1) {
            if let Ok(v) = entry {
                let path = v.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "pcg" {
                        if let Some(name) = v.file_name().to_str() {
                            self.graph_files.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    fn save_graph(&mut self) {
        if self.graph_name.is_empty() {
            return;
        }
        let path_str = self
            .working_directory
            .join(&self.graph_name)
            .to_string_lossy()
            .to_string();
        if let Err(e) = save_graph_file(&path_str, &self.graph) {
            eprintln!("Failed to save graph: {e}");
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleKeybinds => {
                self.show_keybinds = !self.show_keybinds;
                Task::none()
            }
            Message::ChangeTheme(theme) => {
                self.config.theme = theme;
                self.save_config();
                Task::none()
            }
            Message::OpenDirectoryPicker => {
                let folder = FileDialog::new().pick_folder();
                if let Some(v) = folder {
                    self.working_directory = v.clone();
                    self.config.last_opened_directory = v;
                    self.save_config();
                    self.update_graph_list();
                };
                Task::none()
            }
            Message::ChangeScreen(sc) => {
                match self.scene {
                    Screen::Settings => {
                        self.save_config();
                    }
                    _ => {}
                }
                self.scene = sc;
                Task::none()
            }
            Message::LoadGraph(graph_name) => {
                self.save_graph();
                let path_str = self
                    .working_directory
                    .join(&graph_name)
                    .to_string_lossy()
                    .to_string();
                match load_graph_file(path_str.as_str()) {
                    Some(graph) => {
                        self.graph = graph;
                        self.graph_name = graph_name;
                        self.canvas.graph = self.graph.clone(); //Clear canvas
                    }
                    None => {
                        print!("No file found at: {}", path_str)
                    }
                }
                Task::none()
            }
            Message::DeleteGraph(graph_name) => {
                delete_graph_file(&self.working_directory, &graph_name).unwrap();
                self.update_graph_list();
                Task::none()
            }
            Message::CloseNodePicker => {
                self.node_picker.visible = false;
                Task::none()
            }
            Message::ClosePinInputEditor => {
                self.pin_input_editor.visible = false;
                Task::none()
            }
            Message::SetMousePosition(point) => {
                self.mouse_position = point;
                if self.is_draging_node {
                    if let Some(node) = node_from_id(&mut self.graph, self.dragging_node) {
                        let world_mouse = untp(self.mouse_position, self.canvas.transform);
                        let p = world_mouse - self.mouse_node_offset;
                        node.position = (p.x, p.y);
                        self.canvas.graph = self.graph.clone();
                    }
                }
                Task::none()
            }

            Message::AddNewGraph => {
                let mut name = self.new_graph_name.trim().to_string();
                if name.is_empty() {
                    return Task::none();
                }
                if !name.ends_with(".pcg") {
                    name.push_str(".pcg");
                }
                self.graph_name = name;
                self.graph = PCGGraph::default();
                self.new_graph_name = String::default();
                self.save_graph();
                self.update_graph_list();
                Task::none()
            }

            Message::SaveGraph => {
                self.save_graph();
                Task::none()
            }

            Message::NewGraphNameUpdate(s) => {
                self.new_graph_name = s;
                Task::none()
            }
            Message::NodePickerSearch(text) => {
                self.node_picker.search_bar_text = text;
                Task::none()
            }
            Message::SelectNodeType(node_type) => {
                let world_pos = untp(self.mouse_position, self.canvas.transform);
                self.graph
                    .nodes
                    .push(PCGNode::new(node_type, (world_pos.x, world_pos.y)));
                self.node_picker.visible = false;
                self.canvas.graph = self.graph.clone();
                Task::none()
            }
            Message::LeftClickCanvas => {
                let mut connection_to_make: Option<(Uuid, Uuid)> = None;
                let mut pins_to_disconect: Vec<Uuid> = vec![];
                let mut canvas_update: Option<(Point, bool, Color, Uuid)> = None;
                let mut node_to_add: Option<PCGNode> = None;
                let mut nodes_to_remove: Vec<Uuid> = vec![];

                // Track state mutations to perform AFTER the loop finishes
                let mut next_dragging_node: Option<Uuid> = None;
                let mut next_mouse_offset: Option<Vector> = None;
                let mut next_is_dragging: Option<bool> = None;

                let world_mouse = untp(self.mouse_position, self.canvas.transform);

                'outer: for node in &self.graph.nodes {
                    //Check every node
                    if node.get_drag_zone().contains(world_mouse) && !self.is_draging_node {
                        //--------------------------------------
                        // Dragzone logic for when not dragging
                        //--------------------------------------

                        //Deletion logic
                        if self.deleteing {
                            // Collect IDs instead of references (&Pin) to keep the compiler happy
                            for p in node.inputs.iter().chain(node.outputs.iter()) {
                                pins_to_disconect.push(p.id);
                            }
                            nodes_to_remove.push(node.id);
                        }
                        //Duplication logic
                        else if self.modf {
                            let new_node = PCGNode::new(node.node_type, node.position);
                            next_mouse_offset =
                                Some(world_mouse - Point::new(node.position.0, node.position.1));
                            next_dragging_node = Some(new_node.id);
                            next_is_dragging = Some(true);
                            node_to_add = Some(new_node);
                            break 'outer;
                        }
                        //Begin Dragging Logic
                        else {
                            next_mouse_offset =
                                Some(world_mouse - Point::new(node.position.0, node.position.1));
                            next_dragging_node = Some(node.id);
                            next_is_dragging = Some(true);
                            break 'outer;
                        }
                    } else if self.is_draging_node {
                        //End dragging logic
                        next_is_dragging = Some(false);
                        break 'outer;
                    } else if self
                        //Were not in the drag zone but we are probably in the node
                        .mouse_position
                        .distance(node.get_point(self.canvas.transform))
                        <= 1000.0
                    {
                        for (i, pin) in node.inputs.iter().enumerate() {
                            //For every single input pin
                            let pin_position = node.get_input_position(i);
                            if self
                                .mouse_position
                                .distance(tp(pin_position, self.canvas.transform))
                                <= 5.0
                            //Check the distance
                            {
                                //Conection logic
                                if self.canvas.connecting && !self.canvas.is_input {
                                    if let Some(last_pin) =
                                        pin_from_uuid_immut(&self.graph, self.canvas.last_pin)
                                    {
                                        if pin.node_id != last_pin.id {
                                            connection_to_make =
                                                Some((pin.id, self.canvas.last_pin));
                                        }
                                    }
                                } else {
                                    canvas_update = Some((
                                        pin_position,
                                        true,
                                        pin.data_type.get_color(),
                                        pin.id,
                                    ));
                                }
                                break 'outer;
                            }
                        }
                        for (i, pin) in node.outputs.iter().enumerate() {
                            //For every single output pin pin
                            let pin_position = node.get_output_position(i);
                            if self
                                .mouse_position
                                .distance(tp(pin_position, self.canvas.transform))
                                <= 5.0
                            //Check the distance
                            {
                                if self.canvas.connecting && self.canvas.is_input {
                                    if let Some(last_pin) =
                                        pin_from_uuid_immut(&self.graph, self.canvas.last_pin)
                                    {
                                        if pin.node_id != last_pin.id {
                                            connection_to_make =
                                                Some((pin.id, self.canvas.last_pin));
                                        }
                                    }
                                } else {
                                    canvas_update = Some((
                                        pin_position,
                                        false,
                                        pin.data_type.get_color(),
                                        pin.id,
                                    ));
                                }
                                break 'outer;
                            }
                        }
                    }
                }

                // Write Phase: Apply delayed loop mutations safely now that the immutable borrow is dead
                if let Some(offset) = next_mouse_offset {
                    self.mouse_node_offset = offset;
                }
                if let Some(drag_id) = next_dragging_node {
                    self.dragging_node = drag_id;
                }
                if let Some(dragging) = next_is_dragging {
                    self.is_draging_node = dragging;
                }

                // Resolve connections
                if let Some((input_pin_id, last_pin_id)) = connection_to_make {
                    let mut target_pin: Option<&mut Pin> = None;
                    let mut source_pin: Option<&mut Pin> = None;

                    for node in &mut self.graph.nodes {
                        for pin in node.inputs.iter_mut().chain(node.outputs.iter_mut()) {
                            if pin.id == last_pin_id {
                                target_pin = Some(pin);
                            } else if pin.id == input_pin_id {
                                source_pin = Some(pin);
                            }
                        }
                    }

                    if let (Some(source), Some(target)) = (source_pin, target_pin) {
                        connect_pins(source, target);
                    }
                }

                if let Some(node) = node_to_add {
                    self.graph.nodes.push(node);
                }

                if let Some((point, is_input, color, last_id)) = canvas_update {
                    self.canvas.connecting = true;
                    self.canvas.connection_point = point;
                    self.canvas.is_input = is_input;
                    self.canvas.drawing_color = color;
                    self.canvas.last_pin = last_id;
                } else {
                    self.canvas.connecting = false;
                }

                for p_id in pins_to_disconect {
                    if let Some(pin) = pin_from_uuid(&mut self.graph, p_id) {
                        if let Some(connect_id) = pin.connection {
                            if let Some(connection) = pin_from_uuid(&mut self.graph, connect_id) {
                                connection.connection = None
                            }
                        }
                    }
                }

                let mut remove_index: Option<usize> = None;
                let mut g_clone = self.graph.clone();

                for node_id in nodes_to_remove {
                    if let Some(rem_node) = node_from_id(&mut g_clone, node_id) {
                        for (i, node) in self.graph.nodes.iter().enumerate() {
                            if node.id == rem_node.id {
                                remove_index = Some(i)
                            }
                        }
                    }
                }

                if let Some(id_to_remove) = remove_index {
                    self.graph.nodes.remove(id_to_remove);
                }

                self.canvas.graph = self.graph.clone();
                Task::none()
            }

            Message::RightClickCanvas => {
                let world_mouse = untp(self.mouse_position, self.canvas.transform);
                let mut opened_pin_input = false;

                self.pin_input_editor.data = None; // clear stale state every click

                'find_pin: for node in &self.graph.nodes {
                    if self
                        .mouse_position
                        .distance(node.get_point(self.canvas.transform))
                        <= 1000.0
                    {
                        for (i, pin) in node.inputs.iter().enumerate() {
                            let pin_position = node.get_input_position(i);
                            if world_mouse.distance(pin_position) <= 7.0 {
                                opened_pin_input = true;

                                self.pin_input_editor.data = Some(ValueInput::new(&pin.data_type));

                                if self.pin_input_editor.data.is_some() {
                                    self.pin_input_editor.visible = true;
                                    self.pin_input_editor.position =
                                        tp(pin_position, self.canvas.transform);
                                    self.pin_input_editor.current_pin_uuid = pin.id;
                                }

                                break 'find_pin; // stop scanning once we've matched a pin
                            }
                        }
                    }
                }

                if !opened_pin_input {
                    self.node_picker = NodePicker {
                        visible: true,
                        position: self.mouse_position,
                        search_bar_text: String::default(),
                    };
                    return focus("NodeSearch");
                }

                Task::none()
            }
            Message::Scroll(delta) => {
                let trans = self.canvas.transform;
                match delta {
                    ScrollDelta::Lines { x, y } => {
                        if self.modf {
                            let factor = 1.0 + (y * ZOOM_SENSITIVITY * 16.0) as f32;
                            let new_scale_x = (trans.2 * factor).clamp(0.1, 10.0);
                            let new_scale_y = (trans.3 * factor).clamp(0.1, 10.0);
                            let cx = self.mouse_position.x as f32;
                            let cy = self.mouse_position.y as f32;
                            self.canvas.transform = (
                                //Scale relative to the cursor
                                (cx - (cx - trans.0 as f32) * (new_scale_x / trans.2)) as f32,
                                (cy - (cy - trans.1 as f32) * (new_scale_y / trans.3)) as f32,
                                new_scale_x,
                                new_scale_y,
                            );
                        } else {
                            self.canvas.transform = (
                                trans.0 + (x * SCROLL_SENSITIVIY * 16.0),
                                trans.1 + (y * SCROLL_SENSITIVIY * 16.0),
                                trans.2,
                                trans.3,
                            )
                        }
                    }

                    ScrollDelta::Pixels { x, y } => {
                        if self.modf {
                            let factor = 1.0 + (y * ZOOM_SENSITIVITY) as f32;
                            let new_scale_x = (trans.2 * factor).clamp(0.1, 10.0);
                            let new_scale_y = (trans.3 * factor).clamp(0.1, 10.0);
                            let cx = self.mouse_position.x as f32;
                            let cy = self.mouse_position.y as f32;
                            self.canvas.transform = (
                                //Scale relative to the cursor
                                (cx - (cx - trans.0 as f32) * (new_scale_x / trans.2)) as f32,
                                (cy - (cy - trans.1 as f32) * (new_scale_y / trans.3)) as f32,
                                new_scale_x,
                                new_scale_y,
                            );
                        } else {
                            self.canvas.transform = (
                                trans.0 + (x * SCROLL_SENSITIVIY),
                                trans.1 + (y * SCROLL_SENSITIVIY),
                                trans.2,
                                trans.3,
                            )
                        }
                    }
                }
                Task::none()
            }
            Message::ModifierChanged(b) => {
                self.modf = b;
                Task::none()
            }
            Message::DeleteingChanged(b) => {
                self.deleteing = b;
                Task::none()
            }
            Message::SetPinData(pin_id, row_name, i, string) => {
                if let Some(pin) = pin_from_uuid(&mut self.graph, pin_id) {
                    pin.value_input.inputs.get_mut(&row_name).unwrap()[i].value = string
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().filter_map(|event| match event {
            iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => Some(
                Message::ModifierChanged(modifiers.command() | modifiers.control()),
            ),
            iced::Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => match key.as_ref() {
                keyboard::Key::Character("x") => Some(Message::DeleteingChanged(true)),
                _ => None,
            },
            iced::Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => match key.as_ref() {
                keyboard::Key::Character("x") => Some(Message::DeleteingChanged(false)),
                _ => None,
            },
            _ => None,
        })
    }
}

impl Default for GraphEditor {
    fn default() -> Self {
        Self {
            graph_name: String::new(),
            working_directory: PathBuf::new(),
            graph: PCGGraph::default(),
            canvas: GraphCanvas::new(),
            node_picker: NodePicker {
                visible: (false),
                position: (iced::Point::default()),
                search_bar_text: String::default(),
            },
            scene: Screen::Main,
            pin_input_editor: PinInputEditor {
                visible: false,
                position: iced::Point::default(),
                data: None,
                current_pin_uuid: Uuid::nil(),
            },
            mouse_position: iced::Point::default(),
            modf: false,
            deleteing: false,
            is_draging_node: false,
            mouse_node_offset: Vector::default(),
            dragging_node: Uuid::nil(),
            graph_files: vec![],
            new_graph_name: String::default(),
            config: Config::default(),
            show_keybinds: false,
        }
    }
}

fn main() -> iced::Result {
    let settings = iced::Settings {
        // add bundled font to iced
        fonts: vec![LUCIDE_FONT_BYTES.into()],
        ..Default::default()
    };

    let result = iced::application(
        || {
            let mut state = GraphEditor::default();
            state.load_config(); // Config loads here with your serialized enum
            state.working_directory = state.config.last_opened_directory.clone();
            state.update_graph_list();
            (state, iced::Task::none())
        },
        GraphEditor::update,
        GraphEditor::view,
    )
    .settings(settings)
    .subscription(GraphEditor::subscription)
    .title("Graph Editor")
    .theme(|state: &GraphEditor| state.config.theme.to_iced_theme())
    .run();

    match &result {
        Ok(v) => println!("Application Successfully Launched: {v:?}"),
        Err(e) => println!("Application Error: {e:?}"),
    }

    result
}

#[derive(Debug)]
struct GraphCanvas {
    transform: (f32, f32, f32, f32),
    graph: PCGGraph,
    connecting: bool,
    connection_point: Point,
    is_input: bool,
    last_pin: Uuid,
    drawing_color: Color,
}

impl GraphCanvas {
    pub fn new() -> Self {
        Self {
            transform: (0.0, 0.0, 1.0, 1.0),
            graph: PCGGraph::default(),
            connecting: false,
            connection_point: Point { x: (0.0), y: (0.0) },
            is_input: false,
            drawing_color: Color::WHITE,
            last_pin: Uuid::new_v4(),
        }
    }
}

fn draw_connection(
    frame: &mut canvas::Frame,
    start: Point,
    end: Point,
    color: Color,
    trans: (f32, f32, f32, f32),
) {
    let dx = (end.x - start.x).abs();
    let handle = tf((dx * 0.5 + 80.0).clamp(80.0, 300.0), trans);

    let control1 = Point::new(start.x + handle, start.y);
    let control2 = Point::new(end.x - handle, end.y);

    let path = Path::new(|builder| {
        builder.move_to(start);
        builder.bezier_curve_to(control1, control2, end);
    });

    frame.stroke(
        &path,
        Stroke {
            line_dash: LineDash {
                segments: (&vec![tf(6.0, trans)]),
                offset: (0),
            },
            width: tf(3.0, trans),
            ..Default::default()
        }
        .with_color(color),
    );
}

impl canvas::Program<Message> for GraphCanvas {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // Request a redraw on every event so cursor position stays fresh
        Some(canvas::Action::request_redraw())
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // Don't use the cache here — it won't redraw for cursor movement
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let addx = frame.width() / DOT_DENSITY;
        let addy = frame.height() / DOT_DENSITY;

        let mut x = self.transform.0 % addx;
        let mut y = self.transform.1 % addy;

        while y < frame.height() {
            while x < frame.width() {
                frame.fill(
                    &canvas::Path::circle(Point { x, y }, DOT_SIZE),
                    Color::from_rgba(1.0, 1.0, 1.0, 0.1),
                );
                x += addx;
            }
            y += addy;
            x = self.transform.0 % addx;
        }

        //Draws each node and its connections
        for node in self.graph.nodes.iter() {
            node.draw_on_frame(&mut frame, self.transform);
            for (i, input) in node.inputs.iter().enumerate() {
                if let Some(connection) = input.connection {
                    if let Some((start_i, start)) =
                        index_and_pin(&mut self.graph.clone(), connection)
                    {
                        let end = node.get_input_position(i);
                        if let Some(start_node) =
                            node_from_id(&mut self.graph.clone(), start.node_id)
                        {
                            draw_connection(
                                &mut frame,
                                tp(start_node.get_output_position(start_i), self.transform),
                                tp(end, self.transform),
                                start.data_type.get_color(),
                                self.transform,
                            );
                        }
                    }
                }
            }
        }

        //Draw the line when the user is activly connecting points
        if self.connecting {
            let (start, end) = if self.is_input {
                (
                    cursor.position_in(bounds).unwrap_or_default(),
                    tp(self.connection_point, self.transform),
                )
            } else {
                (
                    tp(self.connection_point, self.transform),
                    cursor.position_in(bounds).unwrap_or_default(),
                )
            };

            draw_connection(&mut frame, start, end, self.drawing_color, self.transform);
        }

        vec![frame.into_geometry()]
    }
}
