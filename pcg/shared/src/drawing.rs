pub mod drawing {
    use crate::DataType;
    use crate::DataType::*;
    use crate::PCGNode;
    use crate::{tf, tp, transform_path};

    use iced::{
        Color, Point, Rectangle, Size, Vector, alignment,
        border::Radius,
        widget::canvas::{
            self, Frame, Stroke, Style, Text,
            path::lyon_path::{geom::Angle, math::Transform},
        },
    };

    use std::f32::consts::SQRT_2;
    use std::ops::Sub;

    //Constants
    const NODEWIDTH: f32 = 250.0; //Width of the node

    const NODE_DRAG_HIGHT: f32 = 45.0; //Hight of the drag zone
    const NODE_DRAG_OFFSET: f32 = 2.5; //Padding around the drag zone

    const PIN_DISTANCE: f32 = 20.0; //The seperation between pins, affects how tall nodes are

    //Every data type has a color and shape to help users distinguish them
    impl DataType {
        pub fn get_color(&self) -> Color {
            match self {
                Int | IntArray => Color {
                    r: (0.0),
                    g: (0.7),
                    b: (1.0),
                    a: (1.0),
                },
                Float | FloatArray | FloatRange => Color {
                    r: (0.0),
                    g: (1.0),
                    b: (0.7),
                    a: (1.0),
                },
                DataType::Point | PointArray => Color {
                    r: (1.0),
                    g: (1.0),
                    b: (1.0),
                    a: (1.0),
                },
                DataType::Transform | TransformRange => Color {
                    r: (0.9),
                    g: (0.6),
                    b: (0.2),
                    a: (1.0),
                },
                DataType::Position | PositionRange => Color {
                    r: (0.9),
                    g: (0.2),
                    b: (0.2),
                    a: (1.0),
                },
                DataType::Rotation | RotationRange => Color {
                    r: (0.2),
                    g: (0.2),
                    b: (0.9),
                    a: (1.0),
                },
                DataType::Scale | ScaleRange => Color {
                    r: (0.2),
                    g: (0.9),
                    b: (0.2),
                    a: (1.0),
                },
                _ => Color {
                    r: (0.5),
                    g: (0.5),
                    b: (0.5),
                    a: (1.0),
                },
            }
        }

        fn get_shape(&self) -> Vec<canvas::Path> {
            match self {
                Int | Float => vec![
                    canvas::Path::rounded_rectangle(
                        Point { x: (0.0), y: (0.0) },
                        Size {
                            width: (8.0),
                            height: (8.0),
                        },
                        Radius::from(2.0),
                    )
                    .transform(&Transform::rotation(Angle::degrees(45.0)))
                    .transform(&Transform::translation(
                        0.0,
                        -4.0 * SQRT_2, //This is offseting by 1/2 the diagonal of the square
                    )),
                ],
                FloatRange | TransformRange | PositionRange | RotationRange | ScaleRange => {
                    vec![canvas::Path::rounded_rectangle(
                        Point {
                            x: (-6.0),
                            y: (-2.5),
                        },
                        Size {
                            width: (12.0),
                            height: (5.0),
                        },
                        Radius::from(2.0),
                    )]
                }
                IntArray | FloatArray | PointArray | MeshArray | SplineArray => {
                    let p1 = canvas::Path::rounded_rectangle(
                        Point { x: (0.0), y: (0.0) },
                        Size {
                            width: (4.0),
                            height: (4.0),
                        },
                        Radius::from(1),
                    )
                    .transform(&Transform::rotation(Angle::degrees(45.0)))
                    .transform(&Transform::translation(
                        -3.0,
                        -(4.0 * SQRT_2) + 5.0, //This is offseting by 1/2 the diagonal of the square
                    ));
                    let p2 = p1.clone().transform(&Transform::translation(6.0, 0.0));
                    let p3 = p1.clone().transform(&Transform::translation(0.0, -6.0));
                    let p4 = p2.clone().transform(&Transform::translation(0.0, -6.0));
                    vec![p1, p2, p3, p4]
                }
                _ => vec![
                    canvas::Path::circle(Point::default(), 5.0)
                        .transform(&Transform::translation(0.0, 0.0)),
                ],
            }
        }
    }

    impl PCGNode {
        ///Gets the position for the specified input index
        pub fn get_input_position(&self, index: usize) -> Point {
            Point {
                x: self.position.0,
                y: self.position.1 + (PIN_DISTANCE * (index + 1) as f32) + NODE_DRAG_HIGHT,
            }
        }

        ///Gets the position for the specified output index
        pub fn get_output_position(&self, index: usize) -> Point {
            Point {
                x: self.position.0 + NODEWIDTH,
                y: self.position.1 + (20.0 * (index + 1) as f32) + NODE_DRAG_HIGHT,
            }
        }

        ///Gets the rectangle that is the dragable top bar of a node
        pub fn get_drag_zone(&self) -> Rectangle {
            let point =
                Point::from(self.position) + Vector::new(NODE_DRAG_OFFSET, NODE_DRAG_OFFSET);
            let scale = (
                NODEWIDTH - (NODE_DRAG_OFFSET * 2.0),
                NODE_DRAG_HIGHT - (NODE_DRAG_OFFSET * 2.0),
            );
            Rectangle {
                x: point.x,
                y: point.y,
                width: scale.0,
                height: scale.1,
            }
        }

        ///Draws the node on a frame
        pub fn draw_on_frame(&self, frame: &mut Frame, trans: (f32, f32, f32, f32)) {
            let bg = transform_path(
                canvas::Path::rounded_rectangle(
                    Point {
                        x: self.position.0,
                        y: self.position.1,
                    },
                    //Width is controled by NODEWIDTH, Hight is controled by PIN_DISTANCE
                    Size::new(
                        NODEWIDTH,
                        NODE_DRAG_HIGHT
                            + (NODE_DRAG_OFFSET * 2.0)
                            + (PIN_DISTANCE * (self.inputs.len() + 2) as f32),
                    ),
                    Radius::from(10),
                ),
                trans,
            );
            frame.fill(&bg, Color::BLACK);
            frame.stroke(
                &bg,
                Stroke {
                    style: Style::Solid(Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.3,
                        a: 1.0,
                    }),
                    width: tf(2.0, trans),
                    ..Default::default()
                },
            );

            let drag_zone = self.get_drag_zone();

            frame.fill(
                &&transform_path(
                    canvas::Path::rounded_rectangle(
                        drag_zone.position(),
                        drag_zone.size(),
                        Radius::from(4),
                    ),
                    trans,
                ),
                Color::from_rgba(0.6, 0.7, 1.0, 0.2),
            );

            let node_name_text = canvas::Text {
                content: self.node_type.to_string(),
                position: tp(
                    Point {
                        x: self.position.0 + (NODEWIDTH / 2.0),
                        y: self.position.1 + 5.0,
                    },
                    trans,
                ),
                size: tf(20.0, trans).into(),
                color: Color::WHITE,
                align_x: iced::widget::text::Alignment::Center,
                ..Default::default()
            };

            frame.fill_text(node_name_text);

            //Draw each input
            for (i, input) in self.inputs.iter().enumerate() {
                let inp_pos = tp(self.get_input_position(i), trans);
                let inp_scale = tf(1.0, trans);

                let color = input.data_type.get_color();

                for pth in input.data_type.get_shape() {
                    let tpth = pth
                        .transform(&Transform::scale(inp_scale, inp_scale))
                        .transform(&Transform::translation(inp_pos.x, inp_pos.y));
                    frame.fill(&tpth, color);
                }

                frame.fill_text(Text {
                    content: input.name.to_owned(),
                    color: input.data_type.get_color(),
                    size: tf(15.0, trans).into(),
                    position: inp_pos
                        + Vector {
                            x: tf(8.0, trans),
                            y: 0.0,
                        },
                    align_y: alignment::Vertical::Center,
                    ..Default::default()
                });
            }

            //Draw each output
            for (i, output) in self.outputs.iter().enumerate() {
                let out_pos = tp(self.get_output_position(i), trans);
                let out_scale = tf(1.0, trans);
                let color = output.data_type.get_color();

                for pth in output.data_type.get_shape() {
                    let tpth = pth
                        .transform(&Transform::scale(out_scale, out_scale))
                        .transform(&Transform::translation(out_pos.x, out_pos.y));
                    frame.fill(&tpth, color);
                }

                frame.fill_text(Text {
                    content: output.name.to_owned(),
                    color: output.data_type.get_color(),
                    size: tf(15.0, trans).into(),
                    position: out_pos.sub(iced::Vector {
                        x: tf(8.0, trans),
                        y: 0.0,
                    }),
                    align_y: alignment::Vertical::Center,
                    align_x: iced::widget::text::Alignment::Right,
                    ..Default::default()
                });
            }
        }
    }
}
