pub mod value_input {
    use std::collections::HashMap;

    use crate::{DataType, EvalError, Floatable, PCGPoint, PinValue};
    use glam::{Quat, vec3a};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ValueInput {
        data_type: DataType,
        pub inputs: HashMap<String, Vec<TextInput>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TextInput {
        pub placeholder: String,
        pub value: String,
    }

    impl From<&str> for TextInput {
        fn from(value: &str) -> Self {
            TextInput {
                placeholder: String::from(value),
                value: String::default(),
            }
        }
    }

    fn val_hash<const N: usize>(s: [&str; N]) -> HashMap<String, Vec<TextInput>> {
        HashMap::from([(String::from("Value"), s.iter().map(|&v| v.into()).collect())])
    }

    impl ValueInput {
        pub fn inputs_only(&self) -> HashMap<String, Vec<String>> {
            self.inputs
                .clone()
                .into_iter()
                .map(|(row_name, text_inputs)| {
                    let text_only = text_inputs.iter().map(|ti| ti.value.clone()).collect();
                    (row_name, text_only)
                })
                .collect()
        }
        pub fn to_pin(&self) -> Result<PinValue, EvalError> {
            let inputs: HashMap<String, Vec<String>> = self.inputs_only();
            let val: Vec<String> = inputs["Value"].clone();
            return match self.data_type {
                DataType::Int => match val[0].parse() {
                    Ok(v) => Ok(PinValue::Int(v)),
                    Err(_) => Err(EvalError::MissingInput {
                        node: (Uuid::default()),
                        pin: ("Int".into()),
                    }),
                },
                DataType::Float => match val[0].parse() {
                    Ok(v) => Ok(PinValue::Float(v)),
                    Err(_) => Err(EvalError::MissingInput {
                        node: (Uuid::default()),
                        pin: ("Float".into()),
                    }),
                },
                DataType::FloatRange => {
                    let v: Vec<f32> = val.to_float()?;
                    Ok(PinValue::FloatRange(v[0], v[1]))
                }
                DataType::Bool => match val[0].parse::<bool>() {
                    Ok(v) => Ok(PinValue::Bool(bool::from(v))),
                    Err(_) => Err(EvalError::MissingInput {
                        node: (Uuid::default()),
                        pin: ("Bool".into()),
                    }),
                },
                DataType::TransformRange => {
                    let fposition: Vec<f32> = inputs["From Position"].to_float()?;
                    let frotation: Vec<f32> = inputs["From Rotation"].to_float()?;
                    let fscale: Vec<f32> = inputs["From Scale"].to_float()?;
                    let tposition: Vec<f32> = inputs["To Position"].to_float()?;
                    let trotation: Vec<f32> = inputs["To Rotation"].to_float()?;
                    let tscale: Vec<f32> = inputs["To Scale"].to_float()?;

                    Ok(PinValue::TransformRange(
                        PCGPoint {
                            position: vec3a(fposition[0], fposition[1], fposition[2]),
                            rotation: Quat::from_euler(
                                glam::EulerRot::XYZ,
                                frotation[0],
                                frotation[1],
                                frotation[2],
                            ),
                            scale: vec3a(fscale[0], fscale[1], fscale[2]),
                            ..Default::default()
                        },
                        PCGPoint {
                            position: vec3a(tposition[0], tposition[1], tposition[2]),
                            rotation: Quat::from_euler(
                                glam::EulerRot::XYZ,
                                trotation[0],
                                trotation[1],
                                trotation[2],
                            ),
                            scale: vec3a(tscale[0], tscale[1], tscale[2]),
                            ..Default::default()
                        },
                    ))
                }
                DataType::PositionRange => {
                    let from: Vec<f32> = inputs["From"].to_float()?;
                    let to: Vec<f32> = inputs["To"].to_float()?;
                    Ok(PinValue::PositionRange(
                        vec3a(from[0], from[1], from[2]),
                        vec3a(to[0], to[1], to[2]),
                    ))
                }
                DataType::Position => {
                    let v: Vec<f32> = val.to_float()?;
                    Ok(PinValue::Position(vec3a(v[0], v[1], v[2])))
                }
                DataType::Scale => {
                    let v: Vec<f32> = val.to_float()?;
                    Ok(PinValue::Scale(vec3a(v[0], v[1], v[2])))
                }
                DataType::RotationRange => {
                    let from: Vec<f32> = inputs["From"]
                        .to_float()?
                        .iter()
                        .map(|f| f.to_radians())
                        .collect();
                    let to: Vec<f32> = inputs["To"]
                        .to_float()?
                        .iter()
                        .map(|f| f.to_radians())
                        .collect();
                    Ok(PinValue::RotationRange(
                        Quat::from_euler(glam::EulerRot::XYZ, from[0], from[1], from[2]),
                        Quat::from_euler(glam::EulerRot::XYZ, to[0], to[1], to[2]),
                    ))
                }
                DataType::Rotation => {
                    let v: Vec<f32> = val.to_float()?.iter().map(|f| f.to_radians()).collect();
                    Ok(PinValue::Rotation(Quat::from_euler(
                        glam::EulerRot::XYZ,
                        v[0],
                        v[1],
                        v[2],
                    )))
                }
                DataType::ScaleRange => {
                    let from: Vec<f32> = inputs["From"].to_float()?;
                    let to: Vec<f32> = inputs["To"].to_float()?;
                    Ok(PinValue::ScaleRange(
                        vec3a(from[0], from[1], from[2]),
                        vec3a(to[3], to[4], to[5]),
                    ))
                }
                _ => Ok(PinValue::Int(0)),
            };
        }

        pub fn new(dt: &DataType) -> Self {
            Self {
                inputs: match dt {
                    DataType::Int | DataType::Float => val_hash(["0"]),
                    DataType::StringData | DataType::Bool => val_hash([""]),
                    DataType::Position | DataType::Rotation | DataType::Scale => {
                        val_hash(["x", "y", "z"])
                    }
                    DataType::Transform => HashMap::from([
                        ("Position".into(), vec!["x".into(), "y".into(), "z".into()]),
                        ("Rotation".into(), vec!["x".into(), "y".into(), "z".into()]),
                        ("Scale".into(), vec!["x".into(), "y".into(), "z".into()]),
                    ]),
                    DataType::FloatRange => HashMap::from([
                        ("From".into(), vec!["0.0".into()]),
                        ("To".into(), vec!["0.0".into()]),
                    ]),
                    DataType::PositionRange | DataType::RotationRange | DataType::ScaleRange => {
                        val_hash(["x", "y", "z"])
                    }
                    DataType::TransformRange => HashMap::from([
                        (
                            "From Position".into(),
                            vec!["x".into(), "y".into(), "z".into()],
                        ),
                        (
                            "From Rotation".into(),
                            vec!["x".into(), "y".into(), "z".into()],
                        ),
                        (
                            "From Scale".into(),
                            vec!["x".into(), "y".into(), "z".into()],
                        ),
                        (
                            "To Position".into(),
                            vec!["x".into(), "y".into(), "z".into()],
                        ),
                        (
                            "To Rotation".into(),
                            vec!["x".into(), "y".into(), "z".into()],
                        ),
                        ("To Scale".into(), vec!["x".into(), "y".into(), "z".into()]),
                    ]),
                    _ => val_hash([""]),
                },
                data_type: dt.clone(),
            }
        }
        pub fn name(&self) -> String {
            return self.data_type.to_string();
        }
    }
}
