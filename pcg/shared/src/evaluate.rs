pub mod eval {
    use crate::{
        FromPin, MeshRef, NoiseData3D, PCGGraph, PCGNodeType, PCGPoint, PinValue,
        PositionRangeValue, PositionValue, ScaleRangeValue, ScaleValue, SplineData,
        node_from_id_immut, pin_from_uuid_immut, pin_value_from_dv, point_grid_xz_from_points,
        sample_spline,
    };

    use glam::{Quat, Vec3A};
    use noise::{NoiseFn, Perlin as PerlinNoiseFn};
    use rand::{RngExt, rng};
    use std::collections::HashMap;
    use uuid::Uuid;

    #[derive(Debug)]
    pub enum EvalError {
        MissingInput { node: Uuid, pin: String },
        Cycle,
        TypeMismatch { node: Uuid, expected: &'static str },
    }

    fn topo_sort(graph: &PCGGraph) -> Result<Vec<Uuid>, EvalError> {
        let mut in_degree: HashMap<Uuid, usize> = graph.nodes.iter().map(|n| (n.id, 0)).collect();
        let mut dependents: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

        for node in &graph.nodes {
            for pin in &node.inputs {
                if let Some(src_id) = pin.connection {
                    if let Some(src_pin) = pin_from_uuid_immut(graph, src_id) {
                        *in_degree.get_mut(&node.id).unwrap() += 1;
                        dependents.entry(src_pin.node_id).or_default().push(node.id);
                    }
                }
            }
        }
        let mut queue: Vec<Uuid> = in_degree
            .iter()
            .filter(|&(_, &d)| d == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut order = Vec::with_capacity(graph.nodes.len());

        while let Some(id) = queue.pop() {
            order.push(id);
            for &dep in dependents.get(&id).into_iter().flatten() {
                let e = in_degree.get_mut(&dep).unwrap();
                *e -= 1;
                if *e == 0 {
                    queue.push(dep);
                }
            }
        }

        if order.len() != graph.nodes.len() {
            return Err(EvalError::Cycle);
        }
        Ok(order)
    }

    pub trait GraphHost {
        fn spawn_meshes(&mut self, meshes: &mut Vec<MeshRef>, points: &[PCGPoint]);
        fn snap_points_to_surface(
            &mut self,
            points: &Vec<PCGPoint>,
            from_height: f32,
            distance: f32,
        ) -> Vec<PCGPoint>;
        fn get_splines_from_indexes(&mut self, is: &Vec<usize>) -> Vec<SplineData>;
        fn get_meshs_from_indexes(&mut self, is: &Vec<usize>) -> Vec<MeshRef>;
        fn gprint(&mut self, txt: String);
    }

    pub fn get_input<T: FromPin>(
        inputs: &HashMap<String, PinValue>,
        name: &str,
        node_id: Uuid,
        node_type: PCGNodeType,
    ) -> Result<T, EvalError> {
        let value = inputs.get(name).cloned();
        let found_desc = value
            .as_ref()
            .map(|v| format!("{:?}", v)) // or a variant-name helper
            .unwrap_or_else(|| "<missing>".into());
        value.and_then(T::from_pin).ok_or(EvalError::TypeMismatch {
            node: node_id,
            expected: format!(
                "{} — pin '{}' on {:?} had {}",
                T::NAME,
                name,
                node_type,
                found_desc
            )
            .leak(),
        })
    }

    pub fn evaluate(
        graph: &PCGGraph,
        seeds: &HashMap<Uuid, PinValue>, // output-pin-id -> value, for SplineInput/MeshInput nodes
        host: &mut impl GraphHost,
    ) -> Result<(), EvalError> {
        let mut values = seeds.clone();

        for node_id in topo_sort(graph)? {
            let node = node_from_id_immut(graph, node_id).unwrap();

            //Maps the inputs of the NodeType to the actual data stored in the HashMap
            let inputs: HashMap<String, PinValue> = node
                .inputs
                .iter()
                .map(|p| match p.connection {
                    Some(id) => values
                        .get(&id)
                        .map(|value| (p.name.clone(), value.clone()))
                        .ok_or(EvalError::MissingInput {
                            node: node.id,
                            pin: p.name.clone(),
                        }),
                    None => pin_value_from_dv(&p.data_type, &p.dis_values)
                        .map(|value| (p.name.clone(), value)),
                })
                .collect::<Result<HashMap<_, _>, _>>()?;

            macro_rules! get_input {
                ($name:expr) => {
                    get_input(&inputs, $name, node.id, node.node_type)
                };
            }

            match node.node_type {
                PCGNodeType::GetSplines | PCGNodeType::MeshInput | PCGNodeType::FloatInput => {} // already seeded
                PCGNodeType::GetSplineIndexes => {
                    let f_range: (f32, f32) = get_input!("Indexes")?;
                    let splines = host.get_splines_from_indexes(
                        &(f_range.0 as usize..=f_range.1 as usize).collect(),
                    );
                    let p = splines.len();
                    host.gprint(p.to_string());
                    values.insert(node.outputs[0].id, PinValue::SplineArray(splines));
                }
                PCGNodeType::PerlinNoise => {
                    let offset: Vec3A = get_input!("Offset").map(|PositionValue(v)| v)?;
                    let scale: Vec3A = get_input!("Scale").map(|ScaleValue(v)| v)?;
                    let seed: i32 = get_input!("Seed")?;
                    values.insert(
                        node.outputs[0].id,
                        PinValue::Noise3D(NoiseData3D {
                            noise_type: crate::NoiseType::Perlin,
                            scale: scale,
                            offset: offset,
                            seed: seed,
                        }),
                    );
                }
                PCGNodeType::NoiseDensity => {
                    let mut points: Vec<PCGPoint> = get_input!("Points")?;
                    let noise: NoiseData3D = get_input!("Noise")?;
                    let noise_func = match noise.noise_type {
                        crate::NoiseType::Perlin => PerlinNoiseFn::new(noise.seed as u32),
                    };
                    for point in points.iter_mut() {
                        let pos = (point.position + noise.offset) * noise.scale;
                        point.density =
                            noise_func.get([pos.x as f64, pos.y as f64, pos.z as f64]) as f32;
                    }

                    values.insert(node.outputs[0].id, PinValue::PointArray(points));
                }
                PCGNodeType::Distance => {
                    let mut points: Vec<PCGPoint> = get_input!("Points")?;
                    let other_points: Vec<PCGPoint> = get_input!("Distance From")?;
                    host.gprint(other_points.len().to_string());
                    for point in points.iter_mut() {
                        let mut closest_point = other_points[0];

                        for op in &other_points {
                            let point_to_op = op.position.distance(point.position);
                            let point_to_closest = closest_point.position.distance(point.position);

                            if point_to_op < point_to_closest {
                                closest_point = op.clone();
                            }
                        }
                        point.distance = closest_point.position.distance(point.position);
                    }

                    values.insert(node.outputs[0].id, PinValue::PointArray(points));
                }
                PCGNodeType::InvertDensity => {
                    let mut points: Vec<PCGPoint> = get_input!("Points")?;
                    for point in points.iter_mut() {
                        point.density = 1.0 - point.density.clamp(0.0, 1.0);
                    }

                    values.insert(node.outputs[0].id, PinValue::PointArray(points));
                }
                PCGNodeType::NormalizeDensity => {
                    let mut points: Vec<PCGPoint> = get_input!("Points")?;
                    let (min, max): (f32, f32) = points.iter().fold((0.0, 1.0), |(min, max), x| {
                        (min.min(x.density), max.max(x.density))
                    });

                    let range = max - min;

                    points
                        .iter_mut()
                        .for_each(|p| p.density = (p.density - min) / range);

                    values.insert(node.outputs[0].id, PinValue::PointArray(points));
                }
                PCGNodeType::InvertDistance => {
                    let mut points: Vec<PCGPoint> = get_input!("Points")?;
                    for point in points.iter_mut() {
                        point.density = 1.0 - point.density.clamp(0.0, 1.0);
                    }

                    values.insert(node.outputs[0].id, PinValue::PointArray(points));
                }
                PCGNodeType::DistanceToDensity => {
                    let mut points: Vec<PCGPoint> = get_input!("Points")?;
                    let (min, max) = points
                        .iter()
                        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), &x| {
                            (min.min(x.distance.abs()), max.max(x.distance.abs()))
                        }); //I love rust <3
                    let range = max - min;
                    for point in points.iter_mut() {
                        point.density = (point.distance.abs() - min) / range
                    }

                    values.insert(node.outputs[0].id, PinValue::PointArray(points));
                }
                PCGNodeType::DensityFilter => {
                    let mut points: Vec<PCGPoint> = get_input!("Points")?;
                    let range: (f32, f32) = get_input!("Range")?;
                    let outside_of_range: bool = get_input!("Outside of Range?")?;
                    points.retain(|point| {
                        if outside_of_range {
                            point.density < range.0 || point.density > range.1
                        } else {
                            point.density >= range.0 && point.density <= range.1
                        }
                    });
                    values.insert(node.outputs[0].id, PinValue::PointArray(points));
                }
                PCGNodeType::Add => {
                    let a: f32 = get_input!("a")?;
                    let b: f32 = get_input!("b")?;
                    values.insert(node.outputs[0].id, PinValue::Float(a + b));
                }

                PCGNodeType::SplineSampler => {
                    let splines: Vec<SplineData> = get_input!("Splines")?;
                    let density: f32 = get_input!("Sample Density")?;
                    let mut points = vec![];
                    for s in splines {
                        points.append(&mut sample_spline(&s, density));
                    }
                    values.insert(node.outputs[0].id, PinValue::PointArray(points));
                }
                PCGNodeType::PointGridFromSpline => {
                    let splines: Vec<SplineData> = get_input!("Splines")?;
                    let spacing: f32 = get_input!("Spacing")?;
                    let precision: f32 = get_input!("Precision")?;
                    let mut points = vec![];
                    for s in splines {
                        let bound = sample_spline(&s, precision);
                        points.append(&mut point_grid_xz_from_points(&bound, spacing));
                    }
                    values.insert(node.outputs[0].id, PinValue::PointArray(points));
                }

                PCGNodeType::MeshInstancer => {
                    let points: Vec<PCGPoint> = get_input!("Points")?;
                    let mut meshes: Vec<MeshRef> = get_input!("Meshes")?;
                    host.spawn_meshes(&mut meshes, &points);
                }
                PCGNodeType::MeshDensityInstancer => {
                    let mut points: Vec<PCGPoint> = get_input!("Points")?;
                    let mut meshes: Vec<MeshRef> = get_input!("Meshes")?;
                    let mut r = rng();

                    points.retain(|point| point.density > r.random_range(0.0..1.0));

                    host.spawn_meshes(&mut meshes, &points);
                }
                PCGNodeType::SnapToSurface => {
                    let points: Vec<PCGPoint> = get_input!("Points")?;
                    let start_above: f32 = get_input!("Start Above")?; // if TransformRange impls FromPin as a tuple, or add a dedicated impl
                    let distance: f32 = get_input!("Distance")?;

                    let transformed_points: Vec<PCGPoint> =
                        host.snap_points_to_surface(&points, start_above, distance);

                    values.insert(node.outputs[0].id, PinValue::PointArray(transformed_points));
                }
                PCGNodeType::TransfromPoints => {
                    let points: Vec<PCGPoint> = get_input!("Points")?;
                    let trans: PCGPoint = get_input!("Transform")?; // if TransformRange impls FromPin as a tuple, or add a dedicated impl
                    let weight: f32 = get_input!("Weight")?;

                    let transformed_points: Vec<PCGPoint> = points
                        .into_iter()
                        .map(|point| PCGPoint {
                            scale: point.scale.lerp(point.scale * trans.scale, weight),
                            rotation: point
                                .rotation
                                .slerp(point.rotation * trans.rotation, weight),
                            position: point.position.lerp(point.position + trans.position, weight),
                            ..Default::default()
                        })
                        .collect();

                    values.insert(node.outputs[0].id, PinValue::PointArray(transformed_points));
                }
                PCGNodeType::TransformPointsS => {
                    let points: Vec<PCGPoint> = get_input!("Points")?;
                    let position: Vec3A = get_input!("Position").map(|PositionValue(v)| v)?;
                    let scale: Vec3A = get_input!("Scale").map(|ScaleValue(v)| v)?;
                    let rotation: Quat = get_input!("Rotation")?;
                    let weight: f32 = get_input!("Weight")?;

                    let transformed_points: Vec<PCGPoint> = points
                        .into_iter()
                        .map(|point| PCGPoint {
                            scale: point.scale.lerp(point.scale * scale, weight),
                            rotation: point.rotation.slerp(point.rotation * rotation, weight),
                            position: point.position.lerp(point.position + position, weight),
                            ..Default::default()
                        })
                        .collect();

                    values.insert(node.outputs[0].id, PinValue::PointArray(transformed_points));
                }
                PCGNodeType::TransformPointsRange => {
                    let points: Vec<PCGPoint> = get_input!("Points")?;
                    let (low, high): (PCGPoint, PCGPoint) = get_input!("Transform")?; // if TransformRange impls FromPin as a tuple, or add a dedicated impl
                    let weight: f32 = get_input!("Weight")?;

                    let mut rng = rand::rng();
                    let transformed_points: Vec<PCGPoint> = points
                        .into_iter()
                        .map(|point| {
                            let offset_scale = low.scale.lerp(high.scale, rng.random());
                            let offset_rotation = low.rotation.slerp(high.rotation, rng.random());
                            let offset_position = low.position.lerp(high.position, rng.random());

                            PCGPoint {
                                scale: point.scale.lerp(point.scale * offset_scale, weight),
                                rotation: point
                                    .rotation
                                    .slerp(offset_rotation * point.rotation, weight),
                                position: point
                                    .position
                                    .lerp(point.position + offset_position, weight),
                                ..Default::default()
                            }
                        })
                        .collect();

                    values.insert(node.outputs[0].id, PinValue::PointArray(transformed_points));
                }
                PCGNodeType::TransformPointsRangeS => {
                    let points: Vec<PCGPoint> = get_input!("Points")?;
                    let (low_position, high_position): (Vec3A, Vec3A) =
                        get_input!("Position").map(|PositionRangeValue(v)| v)?;
                    let (low_rotation, high_rotation): (Quat, Quat) = get_input!("Rotation")?;
                    let (low_scale, high_scale): (Vec3A, Vec3A) =
                        get_input!("Scale").map(|ScaleRangeValue(v)| v)?;
                    let weight: f32 = get_input!("Weight")?;

                    let mut rng = rand::rng();
                    let transformed_points: Vec<PCGPoint> = points
                        .into_iter()
                        .map(|point| {
                            let offset_scale = low_scale.lerp(high_scale, rng.random());
                            let offset_rotation = low_rotation.slerp(high_rotation, rng.random());
                            let offset_position = low_position.lerp(high_position, rng.random());

                            PCGPoint {
                                scale: point.scale.lerp(point.scale * offset_scale, weight),
                                rotation: point
                                    .rotation
                                    .slerp(offset_rotation * point.rotation, weight),
                                position: point
                                    .position
                                    .lerp(point.position + offset_position, weight),
                                ..Default::default()
                            }
                        })
                        .collect();

                    values.insert(node.outputs[0].id, PinValue::PointArray(transformed_points));
                }
                _ => {}
            }
        }
        Ok(())
    }
}
