mod view {
    use crate::{
        GraphEditor,
        Message::{self, *},
        PCGNodeType,
        Screen::{self, Settings},
        TOP_BAR_BUTTON_PADDING, pin_from_uuid_immut, settings_menu,
    };
    use iced::widget::text;
    use iced::{
        Background, Color, Element, Fill,
        Length::{self, Shrink},
        Padding,
        alignment::Vertical::Bottom,
        widget::{
            Canvas, Column, Row, Theme, button, column, container, mouse_area, row, scrollable,
            space, stack, text::Alignment::Center, text_input,
        },
    };
    use lucide_icons::iced as Icon;

    use convert_case::{Case::Title, Casing};
    use strum::IntoEnumIterator;

    const SIDE_BAR_WIDTH: f32 = 200.0;

    impl GraphEditor {
        pub fn view(&self) -> Element<'_, Message> {
            //Graph files to buttons
            let files = self.graph_files.iter().map(|name| {
                if self.deleteing {
                    button(text(name).size(12))
                        .width(Fill)
                        .on_press(Message::DeleteGraph(name.to_owned()))
                        .into()
                } else if name.to_owned() == self.graph_name {
                    button(text(name).size(12))
                        .width(Fill)
                        .style(button::background)
                        .into()
                } else {
                    button(text(name).size(12))
                        .width(Fill)
                        .on_press(Message::LoadGraph(name.to_owned()))
                        .into()
                }
            });

            let mut c = row![].align_y(Bottom);

            if self.show_keybinds {
                c = c.push(
                    column![
                        text("Keybinds").size(24),
                        scrollable(
                            column![
                                text("Right click to add new node"),
                                text("Left click on the top of node to move it"),
                                text("CMD or CLTRL click on the top of node to duplicate it"),
                                text("Right click on a pin to enter a value"),
                                text("Left click on a pin to connect it"),
                                text("Hold 'X' and click to delete a graph or a node"),
                                text("In order to use your graph, you must save it by clicking the save button")
                            ]
                            .align_x(Center)
                        )
                    ]
                    .align_x(Center),
                );
                c = c.push(space().width(10));
            }

            // Push the button last so it always renders on the far right
            c = c.push(
                button(Icon::icon_keyboard().color(Color::WHITE).size(15))
                    .style(|_theme: &Theme, _status: button::Status| button::Style {
                        background: Some(Background::Color(Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 0.1,
                        })),
                        ..button::Style::default()
                    })
                    .on_press(Message::ToggleKeybinds)
                    .width(Shrink)
                    .height(Shrink),
            );

            //Main Interface
            let base_layer: Element<Message> = column![
                row![
                    button(text(format!(
                        "Working Directory: {}",
                        self.working_directory.to_str().unwrap_or_default()
                    )))
                    .on_press(Message::OpenDirectoryPicker),
                    button(text("Save")).on_press(Message::SaveGraph),
                    space().width(Fill),
                    button(text("Settings")).on_press(Message::ChangeScreen(Settings))
                ]
                .spacing(TOP_BAR_BUTTON_PADDING)
                .padding(TOP_BAR_BUTTON_PADDING),
                row![
                    container(
                        Column::with_children(files)
                            .push(
                                text_input("Add New Graph....", &self.new_graph_name)
                                    .on_input(NewGraphNameUpdate)
                                    .on_submit(Message::AddNewGraph)
                            )
                            .spacing(2)
                    )
                    .style(container::bordered_box)
                    .width(SIDE_BAR_WIDTH)
                    .align_x(iced::Alignment::Center)
                    .padding(5),
                    container(stack![
                        mouse_area(Canvas::new(&self.canvas).width(Fill).height(Fill))
                            .on_right_press(RightClickCanvas)
                            .on_press(LeftClickCanvas)
                            .on_move(Message::SetMousePosition)
                            .on_scroll(Message::Scroll),
                        container(c).align_bottom(Fill).align_right(Fill).padding(5)
                    ])
                    .style(container::bordered_box)
                    .align_bottom(Fill)
                    .align_right(Fill),
                ]
                .spacing(10)
                .padding(Padding {
                    top: 2.0,
                    left: 10.0,
                    right: 10.0,
                    bottom: 10.0
                }),
            ]
            .into();

            //////////////////////
            // Pin Input Editor //
            /////////////////////

            let mut pin_input_column: Column<'_, Message> = column![];
            let pin_id = self.pin_input_editor.current_pin_uuid;

            if let Some(pin_input_data) = &self.pin_input_editor.data {
                if let Some(pin) = pin_from_uuid_immut(&self.graph, pin_id) {
                    pin_input_column = pin_input_column
                        .push(text(pin.name.clone()).size(20))
                        .align_x(Center);
                    let mut i = 0;
                    for datum in pin_input_data {
                        let mut roo: Row<'_, Message> = row![].padding(2);

                        if datum.0 != String::default() {
                            roo = roo.push(
                                text(datum.0.clone())
                                    .width(Length::Fixed(100.0))
                                    .wrapping(text::Wrapping::Word),
                            );
                        }

                        for v in datum.1.iter() {
                            roo = roo.push(
                                text_input(
                                    v.as_str(),
                                    pin.dis_values.get(i).map(String::as_str).unwrap_or(""),
                                )
                                .on_input(move |v| Message::SetPinData(pin_id, i, v)),
                            );
                            i += 1;
                        }

                        pin_input_column = pin_input_column.push(roo);
                    }
                }
            }

            let pin_input_menu: Element<Message> = container(
                mouse_area(
                    container(pin_input_column.spacing(3))
                        .padding(10)
                        .style(container::rounded_box),
                )
                .on_exit(ClosePinInputEditor),
            )
            .padding(Padding {
                top: self.pin_input_editor.position.y,
                left: self.pin_input_editor.position.x + SIDE_BAR_WIDTH,
                bottom: 500.0 - self.pin_input_editor.position.y,
                right: 500.0 - self.pin_input_editor.position.x - SIDE_BAR_WIDTH,
            })
            .into();

            //////////////////////
            //   Node Search   //
            /////////////////////

            let search_query = self.node_picker.search_bar_text.to_lowercase();

            let node_list_buttons = PCGNodeType::iter()
                .filter(|node_type| {
                    search_query.is_empty()
                        || node_type.to_string().to_lowercase().contains(&search_query)
                })
                .map(|node_type| {
                    button(text(node_type.to_string().to_case(Title)))
                        .width(Fill)
                        .padding(5)
                        .on_press(Message::SelectNodeType(node_type))
                        .into()
                });

            let node_menu: Element<Message> = container(
                mouse_area(
                    container(
                        column![
                            text_input("Search for a node...", &self.node_picker.search_bar_text)
                                .on_input(NodePickerSearch)
                                .id("NodeSearch"),
                            iced::widget::scrollable(
                                Column::with_children(node_list_buttons).spacing(5)
                            )
                            .height(Fill)
                        ]
                        .spacing(3),
                    )
                    .padding(10)
                    .style(container::rounded_box)
                    .width(300)
                    .height(200),
                )
                .on_exit(CloseNodePicker),
            )
            .padding(Padding {
                top: self.node_picker.position.y,
                left: self.node_picker.position.x,
                bottom: 0.0,
                right: 0.0,
            })
            .into();

            //////////////////////
            //     Stacking     //
            /////////////////////

            match self.scene {
                Screen::Settings => settings_menu(&self.config),
                _ => {
                    if self.node_picker.visible {
                        stack!(base_layer, node_menu).into()
                    } else if self.pin_input_editor.visible {
                        stack!(base_layer, pin_input_menu).into()
                    } else {
                        base_layer
                    }
                }
            }
        }
    }
}
