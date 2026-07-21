pub mod floatable {
    use crate::EvalError;
    use uuid::Uuid;

    pub trait Floatable {
        fn to_float(&self) -> Result<Vec<f32>, EvalError> {
            return Ok(vec![]);
        }
    }

    impl Floatable for Vec<String> {
        fn to_float(&self) -> Result<Vec<f32>, EvalError> {
            self.iter()
                .map(|s| {
                    s.parse::<f32>().map_err(|_| EvalError::TypeMismatch {
                        node: Uuid::nil(),
                        expected: "float",
                    })
                })
                .collect()
        }
    }
}
