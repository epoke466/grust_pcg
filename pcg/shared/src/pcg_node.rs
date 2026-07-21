pub mod pcg_node {
    use iced::Point;
    use serde::{Deserialize, Serialize};
    use strum_macros::{Display, EnumIter};
    use uuid::Uuid;

    use crate::{DataType::*, PCGGraph, PCGNodeType::*, Pin, tp};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display, Serialize, Deserialize)]
    pub enum PCGNodeType {
        //Samplers
        SplineSampler,
        //MeshSampler,
        PointGridFromSpline,

        //Spawners
        MeshInstancer,

        //Inputs
        FloatInput,
        SplineInput,
        MeshInput,

        //Math
        Add,
        Subtract,
        Multiply,
        Divide,
        Mod,

        //Density
        DistanceDensity,
        //NormalDensity,
        NoiseDensity,
        PerlinNoise,

        //Filters
        AttributeFilter,
        DensityFilter,

        //Point Operations
        TransfromPoints,
        TransformPointsS,
        TransformPointsRange,
        TransformPointsRangeS,
        SnapToSurface,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PCGNode {
        pub id: Uuid,
        pub node_type: PCGNodeType,
        pub inputs: Vec<Pin>,
        pub outputs: Vec<Pin>,
        pub position: (f32, f32),
    }

    impl PCGNode {
        //-------------------------------------------------------------------------------------------------//
        // This is where we match a node type with a node, it will be long, but maybe it will be worth it? //
        // ------------------------------------------------------------------------------------------------//

        pub fn new(node_type: PCGNodeType, position: (f32, f32)) -> Self {
            let node_id = Uuid::new_v4();
            match node_type {
                SplineInput => Self {
                    id: node_id,
                    node_type: node_type,
                    inputs: vec![],
                    outputs: vec![Pin::new("Splines", SplineArray, node_id)],
                    position: position,
                },
                SnapToSurface => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Points", PointArray, node_id),
                        Pin::new("Start Above", Float, node_id),
                        Pin::new("Distance", Float, node_id),
                    ],
                    outputs: vec![Pin::new("Points", PointArray, node_id)],
                    position,
                },
                MeshInput => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![],
                    outputs: vec![Pin::new("Meshes", MeshArray, node_id)],
                    position,
                },
                SplineSampler => Self {
                    id: node_id,
                    node_type: node_type,
                    inputs: vec![
                        Pin::new("Splines", SplineArray, node_id),
                        Pin::new("Sample Density", Float, node_id),
                    ],
                    outputs: vec![Pin::new("Points", PointArray, node_id)],
                    position: position,
                },
                MeshInstancer => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Points", PointArray, node_id),
                        Pin::new("Meshes", MeshArray, node_id),
                    ],
                    outputs: vec![],
                    position,
                },

                TransfromPoints => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Points", PointArray, node_id),
                        Pin::new("Transform", Transform, node_id),
                        Pin::new("Weight", Float, node_id),
                    ],
                    outputs: vec![Pin::new("Points", PointArray, node_id)],
                    position,
                },
                TransformPointsS => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Points", PointArray, node_id),
                        Pin::new("Position", Position, node_id),
                        Pin::new("Rotation", Rotation, node_id),
                        Pin::new("Scale", Scale, node_id),
                        Pin::new("Weight", Float, node_id),
                    ],
                    outputs: vec![Pin::new("Points", PointArray, node_id)],
                    position,
                },
                TransformPointsRange => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Points", PointArray, node_id),
                        Pin::new("Transform", TransformRange, node_id),
                        Pin::new("Weight", Float, node_id),
                    ],
                    outputs: vec![Pin::new("Points", PointArray, node_id)],
                    position,
                },
                TransformPointsRangeS => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Points", PointArray, node_id),
                        Pin::new("Position", PositionRange, node_id),
                        Pin::new("Rotation", RotationRange, node_id),
                        Pin::new("Scale", ScaleRange, node_id),
                        Pin::new("Weight", Float, node_id),
                    ],
                    outputs: vec![Pin::new("Points", PointArray, node_id)],
                    position,
                },

                FloatInput => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![],
                    outputs: vec![Pin::new("Float", Float, node_id)],
                    position,
                },
                PointGridFromSpline => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Splines", SplineArray, node_id),
                        Pin::new("Spacing", Float, node_id),
                        Pin::new("Precision", Float, node_id),
                    ],
                    outputs: vec![Pin::new("Points", PointArray, node_id)],
                    position,
                },
                PerlinNoise => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Offset", Position, node_id),
                        Pin::new("Scale", Scale, node_id),
                        Pin::new("Seed", Int, node_id),
                    ],
                    outputs: vec![Pin::new("Noise", Noise3D, node_id)],
                    position,
                },
                NoiseDensity => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Points", PointArray, node_id),
                        Pin::new("Noise", Noise3D, node_id),
                    ],
                    outputs: vec![Pin::new("Points", PointArray, node_id)],
                    position,
                },
                DensityFilter => Self {
                    id: node_id,
                    node_type,
                    inputs: vec![
                        Pin::new("Points", PointArray, node_id),
                        Pin::new("Range", FloatRange, node_id),
                        Pin::new("Outside of Range?", Bool, node_id),
                    ],
                    outputs: vec![Pin::new("Points", PointArray, node_id)],
                    position,
                },
                _ => {
                    return Self {
                        id: node_id,
                        node_type: node_type,
                        inputs: vec![],
                        outputs: vec![Pin::new("Spline", Spline, node_id)],
                        position: position,
                    };
                }
            }
        }

        pub fn get_point(&self, trans: (f32, f32, f32, f32)) -> Point {
            tp(
                Point {
                    x: (self.position.0),
                    y: (self.position.1),
                },
                trans,
            )
        }
    }
    pub fn node_from_id(grap: &mut PCGGraph, id: Uuid) -> Option<&mut PCGNode> {
        for node in &mut grap.nodes {
            if node.id == id {
                return Some(node);
            }
        }
        None
    }

    pub fn node_from_id_immut(grap: &PCGGraph, id: Uuid) -> Option<&PCGNode> {
        for node in &grap.nodes {
            if node.id == id {
                return Some(node);
            }
        }
        None
    }
}
