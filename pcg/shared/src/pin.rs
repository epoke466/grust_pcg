pub mod pin {
    use crate::{
        DataType::*, EvalError, Floatable, MeshRef, PCGGraph, PCGPoint, PinValue::FloatRange,
        SplineData,
    };
    use glam::{Quat, Vec3A, vec3a};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DataType {
        StringData,
        StringArray,
        Float,
        FloatRange,
        FloatArray,
        Bool,

        Int,
        IntArray,

        Point,
        PointArray,
        Spline,
        SplineArray,
        Mesh,
        MeshArray,

        Transform,
        Position,
        Rotation,
        Scale,

        TransformRange,
        PositionRange,
        RotationRange,
        ScaleRange,

        Noise3D,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Pin {
        pub name: String,
        pub data_type: DataType,
        pub current_position: (f32, f32),
        pub id: Uuid,
        pub connection: Option<Uuid>,
        pub node_id: Uuid,
        pub dis_values: Vec<String>,
        pub dvid: Uuid,
    }

    impl Pin {
        pub fn new(name: &str, data_type: DataType, node_id: Uuid) -> Self {
            //Dis Value Sizing
            let needed = match data_type {
                DataType::Int | DataType::Float | StringData | Bool => 1,
                DataType::FloatRange => 2,
                DataType::Position | DataType::Rotation | DataType::Scale => 3,
                PositionRange | RotationRange | ScaleRange => 6,
                DataType::Transform => 9,
                DataType::TransformRange => 18,
                _ => 0,
            };
            let mut workin = Self {
                name: (String::from(name)),
                data_type,
                current_position: (0.0, 0.0),
                id: (Uuid::new_v4()),
                connection: (None),
                node_id: node_id,
                dis_values: Vec::new(),
                dvid: Uuid::new_v4(),
            };
            while workin.dis_values.len() < needed {
                workin.dis_values.push(String::default());
            }
            return workin;
        }

        pub fn dis_vals_assigned(self) -> bool {
            if !self.dis_values.is_empty() {
                for v in self.dis_values {
                    if v != String::default() {
                        return true;
                    }
                }
            }
            return false;
        }
    }

    pub fn connect_pins(a: &mut Pin, b: &mut Pin) {
        a.connection = Some(b.id);
        b.connection = Some(a.id);
    }

    pub fn pin_from_uuid(grap: &mut PCGGraph, id: Uuid) -> Option<&mut Pin> {
        for node in &mut grap.nodes {
            for pin in &mut node.inputs {
                if pin.id == id {
                    return Some(pin);
                }
            }
            for pin in &mut node.outputs {
                if pin.id == id {
                    return Some(pin);
                }
            }
        }
        None
    }

    pub fn pin_from_uuid_immut(grap: &PCGGraph, id: Uuid) -> Option<&Pin> {
        for node in &grap.nodes {
            for pin in &node.inputs {
                if pin.id == id {
                    return Some(pin);
                }
            }
            for pin in &node.outputs {
                if pin.id == id {
                    return Some(pin);
                }
            }
        }
        None
    }

    pub fn index_and_pin(grap: &mut PCGGraph, id: Uuid) -> Option<(usize, &Pin)> {
        for node in &mut grap.nodes {
            for (i, pin) in &mut node.inputs.iter().enumerate() {
                if pin.id == id {
                    return Some((i, pin));
                }
            }
            for (i, pin) in &mut node.outputs.iter().enumerate() {
                if pin.id == id {
                    return Some((i, pin));
                }
            }
        }
        None
    }

    #[derive(Debug, Clone)]
    pub enum PinValue {
        Float(f32),
        Int(i32),
        Point(PCGPoint),
        PointArray(Vec<PCGPoint>),
        Spline(SplineData),
        SplineArray(Vec<SplineData>),
        Mesh(MeshRef),
        MeshArray(Vec<MeshRef>),
        Transform(PCGPoint),
        TransformRange(PCGPoint, PCGPoint),
        Position(Vec3A),
        Rotation(Quat),
        Scale(Vec3A),
        PositionRange(Vec3A, Vec3A),
        RotationRange(Quat, Quat),
        ScaleRange(Vec3A, Vec3A),
        Noise3D(NoiseData3D),
        Bool(bool),
        FloatRange(f32, f32),
    }

    #[derive(Debug, Clone)]
    pub enum NoiseType {
        Perlin,
    }

    #[derive(Debug, Clone)]
    pub struct NoiseData3D {
        pub noise_type: NoiseType,
        pub scale: Vec3A,
        pub offset: Vec3A,
        pub seed: i32,
    }

    pub trait FromPin: Sized {
        const NAME: &'static str;
        fn from_pin(pv: PinValue) -> Option<Self>;
    }

    //Some meta magic be going on here, ima be fr claude did this part for me,
    // I can't read it at all but its nice to have, and easy to duplicate
    macro_rules! impl_from_pin {
        ($ty:ty, $variant:ident, $name:expr) => {
            impl FromPin for $ty {
                const NAME: &'static str = $name;
                fn from_pin(pv: PinValue) -> Option<Self> {
                    if let PinValue::$variant(v) = pv { Some(v) } else { None }
                }
            }
        };
        ($ty:ty, $variant:ident($($field:ident),+), $name:expr) => {
            impl FromPin for $ty {
                const NAME: &'static str = $name;
                fn from_pin(pv: PinValue) -> Option<Self> {
                    if let PinValue::$variant($($field),+) = pv { Some(($($field),+)) } else { None }
                }
            }
        };
        ($ty:ty, $inner:ty, $variant:ident, $name:expr) => {
            impl FromPin for $ty {
                const NAME: &'static str = $name;
                fn from_pin(pv: PinValue) -> Option<Self> {
                    if let PinValue::$variant(v) = pv { Some(Self(v)) } else { None }
                }
            }
        };
        ($ty:ty, $inner:ty, $variant:ident($($field:ident),+), $name:expr) => {
            impl FromPin for $ty {
                const NAME: &'static str = $name;
                fn from_pin(pv: PinValue) -> Option<Self> {
                    if let PinValue::$variant($($field),+) = pv {
                        Some(Self(($($field),+)))
                    } else {
                        None
                    }
                }
            }
        };
    }
    //(
    // 1. What it's converted into, this is the real data that is bundle (e.g. f32, String),
    // 2. The PinValue it is stored as
    // 3. The name of the pin
    // )
    impl_from_pin!(f32, Float, "Float");
    impl_from_pin!(i32, Int, "Int");
    impl_from_pin!(NoiseData3D, Noise3D, "Noise3D");
    impl_from_pin!(SplineData, Spline, "Spline");
    impl_from_pin!(Vec<PCGPoint>, PointArray, "Points");
    impl_from_pin!(MeshRef, Mesh, "Mesh");
    impl_from_pin!(PCGPoint, Transform, "Transform");
    pub struct PositionValue(pub Vec3A);
    impl_from_pin!(PositionValue, Vec3A, Position, "Position");
    pub struct ScaleValue(pub Vec3A);
    impl_from_pin!(ScaleValue, Vec3A, Scale, "Scale");
    impl_from_pin!(Quat, Rotation, "Rotation");
    impl_from_pin!(
        (PCGPoint, PCGPoint),
        TransformRange(low, high),
        "Transform Range"
    );
    impl_from_pin!(Vec<SplineData>, SplineArray, "Spline Array");
    impl_from_pin!(Vec<MeshRef>, MeshArray, "Mesh Array");

    impl_from_pin!((f32, f32), FloatRange(low, high), "Float Range");
    impl_from_pin!(bool, Bool, "Boolean");

    pub struct PositionRangeValue(pub (Vec3A, Vec3A));
    impl_from_pin!(
        PositionRangeValue,
        (Vec3A, Vec3A),
        PositionRange(low, high),
        "Position Range"
    );
    pub struct ScaleRangeValue(pub (Vec3A, Vec3A));
    impl_from_pin!(
        ScaleRangeValue,
        (Vec3A, Vec3A),
        ScaleRange(low, high),
        "Scale Range"
    );
    impl_from_pin!((Quat, Quat), RotationRange(low, high), "Rotation Range");

    pub fn pin_value_from_dv(ptype: &DataType, dv: &Vec<String>) -> Result<PinValue, EvalError> {
        if !dv.is_empty() {
            return match ptype {
                DataType::Int => match dv[0].parse() {
                    Ok(v) => Ok(PinValue::Int(v)),
                    Err(_) => Err(EvalError::MissingInput {
                        node: (Uuid::default()),
                        pin: ("Unknown".into()),
                    }),
                },
                DataType::Float => match dv[0].parse() {
                    Ok(v) => Ok(PinValue::Float(v)),
                    Err(_) => Err(EvalError::MissingInput {
                        node: (Uuid::default()),
                        pin: ("Unknown".into()),
                    }),
                },
                DataType::FloatRange => {
                    let v: Vec<f32> = dv.to_float()?;
                    Ok(FloatRange(v[0], v[1]))
                }
                DataType::Bool => match dv[0].parse::<bool>() {
                    Ok(v) => Ok(PinValue::Bool(bool::from(v))),
                    Err(_) => Err(EvalError::MissingInput {
                        node: (Uuid::default()),
                        pin: ("Bool".into()),
                    }),
                },
                DataType::TransformRange => {
                    let v: Vec<f32> = dv.to_float()?;
                    // These are the indexes

                    // Position 0, 1, 2
                    // Rotation 3, 4, 5
                    // Scale 6, 7, 8

                    // Position 9, 10, 11
                    // Rotation 12, 13, 14
                    // Scale 15, 16, 17

                    Ok(PinValue::TransformRange(
                        PCGPoint {
                            position: vec3a(v[6], v[7], v[8]),
                            rotation: Quat::from_euler(glam::EulerRot::XYZ, v[3], v[4], v[5]),
                            scale: vec3a(v[0], v[1], v[2]),
                            ..Default::default()
                        },
                        PCGPoint {
                            position: vec3a(v[15], v[16], v[17]),
                            rotation: Quat::from_euler(glam::EulerRot::XYZ, v[12], v[13], v[14]),
                            scale: vec3a(v[9], v[10], v[11]),
                            ..Default::default()
                        },
                    ))
                }
                DataType::PositionRange => {
                    let v: Vec<f32> = dv.to_float()?;
                    Ok(PinValue::PositionRange(
                        vec3a(v[0], v[1], v[2]),
                        vec3a(v[3], v[4], v[5]),
                    ))
                }
                DataType::Position => {
                    let v: Vec<f32> = dv.to_float()?;
                    Ok(PinValue::Position(vec3a(v[0], v[1], v[2])))
                }
                DataType::Scale => {
                    let v: Vec<f32> = dv.to_float()?;
                    Ok(PinValue::Scale(vec3a(v[0], v[1], v[2])))
                }
                DataType::RotationRange => {
                    let v: Vec<f32> = dv.to_float()?.iter().map(|f| f.to_radians()).collect();
                    Ok(PinValue::RotationRange(
                        Quat::from_euler(glam::EulerRot::XYZ, v[0], v[1], v[2]),
                        Quat::from_euler(glam::EulerRot::XYZ, v[3], v[4], v[5]),
                    ))
                }
                DataType::Rotation => {
                    let v: Vec<f32> = dv.to_float()?.iter().map(|f| f.to_radians()).collect();
                    Ok(PinValue::Rotation(Quat::from_euler(
                        glam::EulerRot::XYZ,
                        v[0],
                        v[1],
                        v[2],
                    )))
                }
                DataType::ScaleRange => {
                    let v: Vec<f32> = dv.to_float()?;
                    Ok(PinValue::ScaleRange(
                        vec3a(v[0], v[1], v[2]),
                        vec3a(v[3], v[4], v[5]),
                    ))
                }
                _ => Ok(PinValue::Int(0)),
            };
        } else {
            return Err(EvalError::MissingInput {
                node: (Uuid::default()),
                pin: ("Unknown".into()),
            });
        }
    }
}
