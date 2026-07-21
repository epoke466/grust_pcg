mod defalut {
    use crate::AppTheme;
    use crate::Config;

    impl Default for Config {
        fn default() -> Self {
            Self {
                theme: AppTheme::Dark,
            }
        }
    }
}
