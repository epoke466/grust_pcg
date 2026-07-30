mod defalut {
    use std::path::PathBuf;

    use crate::AppTheme;
    use crate::Config;

    impl Default for Config {
        fn default() -> Self {
            Self {
                theme: AppTheme::Dark,
                last_opened_directory: PathBuf::new(),
            }
        }
    }
}
