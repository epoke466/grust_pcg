pub mod settings {
    use std::path::PathBuf;

    use crate::{GraphEditor, Message, Screen, TOP_BAR_BUTTON_PADDING};
    use iced::Theme::*;
    use iced::widget::container::bordered_box;
    use iced::widget::pick_list;
    use iced::{
        Element, Fill,
        widget::{button, column, container, row, scrollable, space, text},
    };
    use strum::{self, IntoEnumIterator};
    use strum_macros::{self, Display, EnumIter};

    use confy::{load, store};
    use serde::{Deserialize, Serialize};

    const APP_NAME: &'static str = "Graph Editor";
    const CFG_NAME: &'static str = "Settings";

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Config {
        pub theme: AppTheme,
        pub last_opened_directory: PathBuf,
    }

    #[derive(Serialize, Deserialize, Debug, EnumIter, PartialEq, Display, Clone)]
    pub enum AppTheme {
        //Light,
        Dark,
        Dracula,
        Nord,
        //SolarizedLight,
        SolarizedDark,
        //GruvboxLight,
        GruvboxDark,
        //CatppuccinLatte,
        CatppuccinFrappe,
        CatppuccinMacchiato,
        CatppuccinMocha,
        TokyoNight,
        TokyoNightStorm,
        //TokyoNightLight,
        KanagawaWave,
        KanagawaDragon,
        //KanagawaLotus,
        //Moonfly,
        Nightfly,
        Oxocarbon,
        //Ferra,
        //MatchSystem,
    }
    impl AppTheme {
        pub fn to_iced_theme(&self) -> iced::Theme {
            match self {
                //Self::Light => Light,
                Self::Dark => Dark,
                Self::Dracula => Dracula,
                Self::Nord => Nord,
                //Self::SolarizedLight => SolarizedLight,
                Self::SolarizedDark => SolarizedDark,
                //Self::GruvboxLight => GruvboxLight,
                Self::GruvboxDark => GruvboxDark,
                //Self::CatppuccinLatte => CatppuccinLatte,
                Self::CatppuccinFrappe => CatppuccinFrappe,
                Self::CatppuccinMacchiato => CatppuccinMacchiato,
                Self::CatppuccinMocha => CatppuccinMocha,
                Self::TokyoNight => TokyoNight,
                Self::TokyoNightStorm => TokyoNightStorm,
                //Self::TokyoNightLight => TokyoNightLight,
                Self::KanagawaWave => KanagawaWave,
                Self::KanagawaDragon => KanagawaDragon,
                //Self::KanagawaLotus => KanagawaLotus,
                //Self::Moonfly => Moonfly,
                Self::Nightfly => Nightfly,
                Self::Oxocarbon => Oxocarbon,
                //Self::Ferra => Ferra,
                // Self::MatchSystem => match from_system() {
                //     Ok((theme, _resolved, _is_dark)) => theme,
                //     Err(_) => Dark,
                // },
            }
        }
    }
    pub fn settings_menu(cfg: &Config) -> Element<'static, Message> {
        column![
            row![
                space().width(Fill),
                text("Settings").size(32),
                space().width(Fill),
                button(text("Close")).on_press(Message::ChangeScreen(Screen::Main)),
            ]
            .padding(TOP_BAR_BUTTON_PADDING),
            container(
                container(scrollable(column![row![
                    space().width(Fill),
                    text("Theme"),
                    space().width(Fill),
                    pick_list(
                        AppTheme::iter().collect::<Vec<_>>(),
                        Some(cfg.theme.clone()),
                        Message::ChangeTheme
                    ),
                    space().width(Fill),
                ]]))
                .padding(10)
                .width(Fill)
                .height(Fill)
                .style(bordered_box)
            )
            .padding(10)
        ]
        .into()
    }
    impl GraphEditor {
        pub fn load_config(&mut self) {
            match load(APP_NAME, CFG_NAME) {
                Ok(cfg) => self.config = cfg,
                Err(_) => {
                    self.config = Config::default();
                }
            }
        }
        pub fn save_config(&self) {
            store(APP_NAME, CFG_NAME, &self.config).unwrap();
        }
    }
}
