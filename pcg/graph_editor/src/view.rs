mod view {
    use crate::{
        GraphEditor,
        Message::{self, *},
        PCGNodeType,
        Screen::{self, Settings},
        TEXT_SIZE, TITLE_SIZE, TOP_BAR_BUTTON_PADDING, pin_from_uuid_immut, settings_menu,
    };
    use iced::{
        Background, Color, Element, Fill,
        Length::{self, Shrink},
        Padding,
        alignment::Vertical::Bottom,
        widget::{
            Canvas, Column, Row, Theme, button, column, container, mouse_area, row, scrollable,
            space, stack, text, text::Alignment::Center, text_input,
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
                    button(text(name).size(TEXT_SIZE))
                        .width(Fill)
                        .on_press(Message::DeleteGraph(name.to_owned()))
                        .into()
                } else if name.to_owned() == self.graph_name {
                    button(text(name).size(TEXT_SIZE))
                        .width(Fill)
                        .style(button::background)
                        .into()
                } else {
                    button(text(name).size(TEXT_SIZE))
                        .width(Fill)
                        .on_press(Message::LoadGraph(name.to_owned()))
                        .into()
                }
            });

            let mut c = row![].align_y(Bottom);

            if self.show_keybinds {
                c = c.push(
                    column![
                        text("Keybinds").size(TITLE_SIZE),
                        scrollable(
                            column![
                                text("Right click to add new node").size(TEXT_SIZE),
                                text("Left click on the top of node to move it").size(TEXT_SIZE),
                                text("CMD or CLTRL click on the top of node to duplicate it").size(TEXT_SIZE),
                                text("Right click on a pin to enter a value").size(TEXT_SIZE),
                                text("Left click on a pin to connect it").size(TEXT_SIZE),
                                text("Hold 'X' and click to delete a graph or a node").size(TEXT_SIZE),
                                text("In order to use your graph, you must save it by clicking the save button").size(TEXT_SIZE),
                                text("CMD/CTRL + Scroll to zoom in and out").size(TEXT_SIZE)
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
                    button(
                        text(format!(
                            "Working Directory: {}",
                            self.working_directory.to_str().unwrap_or_default()
                        ))
                        .size(TEXT_SIZE)
                    )
                    .on_press(Message::OpenDirectoryPicker),
                    button(text("Save").size(TEXT_SIZE)).on_press(Message::SaveGraph),
                    space().width(Fill),
                    button(text("Settings").size(TEXT_SIZE))
                        .on_press(Message::ChangeScreen(Settings))
                ]
                .spacing(TOP_BAR_BUTTON_PADDING)
                .padding(TOP_BAR_BUTTON_PADDING),
                row![
                    column![
                        container(
                            Column::with_children(files)
                                .push(
                                    text_input("Add New Graph....", &self.new_graph_name)
                                        .on_input(NewGraphNameUpdate)
                                        .on_submit(Message::AddNewGraph)
                                        .size(TEXT_SIZE)
                                )
                                .spacing(2)
                        )
                        .style(container::bordered_box)
                        .width(SIDE_BAR_WIDTH)
                        .padding(5),
                        container(column![
                            text("Paramaters").size(TITLE_SIZE),
                            scrollable(column![text("Bla").size(TEXT_SIZE)])
                        ])
                        .style(container::bordered_box)
                        .width(SIDE_BAR_WIDTH)
                        .padding(5)
                    ]
                    .spacing(5)
                    .align_x(iced::Alignment::Center),
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

            if let Some(pin) = pin_from_uuid_immut(&self.graph, pin_id) {
                if let Some(value_input) = &pin.value_input {
                    pin_input_column = pin_input_column
                        .push(text(value_input.name()).size(TITLE_SIZE))
                        .align_x(Center);

                    for row_name in value_input.inputs.keys() {
                        let mut roo: Row<'_, Message> = row![].padding(2);

                        roo = roo.push(
                            text(row_name)
                                .width(Length::Fixed(100.0))
                                .wrapping(text::Wrapping::Word)
                                .size(TEXT_SIZE),
                        );
                        for (i, text) in value_input.inputs[row_name].iter().enumerate() {
                            roo = roo.push(
                                text_input(&text.placeholder, &text.value)
                                    .size(TEXT_SIZE)
                                    .on_input(move |v| {
                                        Message::SetPinData(pin_id, row_name.clone(), i, v)
                                    }),
                            );
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
                    button(text(node_type.to_string().to_case(Title)).size(TEXT_SIZE))
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
                                .size(TITLE_SIZE)
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
