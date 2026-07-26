use godot::classes::{
    CollisionShape3D, Curve3D, EditorPlugin, IEditorPlugin, Mesh, MultiMesh, MultiMeshInstance3D,
    Path3D, PhysicsRayQueryParameters3D, ProjectSettings, multi_mesh,
};

use glam::{Quat, vec3a};
use godot::prelude::*;
use rand::{
    distr::{Distribution, weighted::WeightedIndex},
    rng,
};
use shared::*;
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

// ── Godot plugin ──────────────────────────────────────────────────────────────
#[derive(GodotClass)]
#[class(tool, init, base=EditorPlugin)]
struct PCGPlugin {
    base: Base<EditorPlugin>,
}

//This is our good boi who does the actuall spawning of meshes
struct GodotHost<'a> {
    zone: &'a mut PCGZone,
    meshes: HashMap<Uuid, Gd<Mesh>>,
}

#[godot_api]
impl PCGPlugin {
    #[func]
    fn open_graph_editor(&mut self) {
        open_graph_editor();
    }
}

#[godot_api]
impl IEditorPlugin for PCGPlugin {
    fn enter_tree(&mut self) {
        let callable = self.base().callable("open_graph_editor");

        self.base_mut()
            .add_tool_menu_item("Open Graph Editor", &callable);

        godot_print!("PLUGIN LOADED");
    }

    fn exit_tree(&mut self) {
        self.base_mut().remove_tool_menu_item("Open Graph Editor");
    }
}

#[gdextension]
unsafe impl ExtensionLibrary for PCGPlugin {}

#[derive(GodotClass)]
#[class(init, tool, base = CollisionShape3D)]
struct SampleZone {
    #[export]
    direction: Vector3,

    base: Base<CollisionShape3D>,
}

#[derive(GodotClass)]
#[class(init, tool, base = Node3D)]
struct PCGZone {
    #[export_tool_button(fn = Self::run, icon = "MainPlay", name = "Run")]
    run: PhantomVar<Callable>,

    #[export_tool_button(fn = Self::open_ge, icon = "file", name = "Open Editor")]
    open_ge: PhantomVar<Callable>,

    #[export(file = "*.pcg")]
    file_path: GString,

    #[export_group(name = "Inputs")]
    #[export]
    splines: Array<Gd<Path3D>>,

    #[export]
    zones: Array<Gd<CollisionShape3D>>,

    #[export]
    spawn_meshes: Array<Gd<Mesh>>,

    #[export]
    float_input: f64,

    base: Base<Node3D>,
}

fn find_graph_editor_executable() -> Option<PathBuf> {
    let project_dir = PathBuf::from(
        ProjectSettings::singleton()
            .globalize_path("res://")
            .to_string(),
    );

    let binary_name = if cfg!(target_os = "windows") {
        "graph_editor.exe"
    } else {
        "graph_editor"
    };
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let os = if cfg!(target_os = "windows") {
        "win"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "lnx"
    };

    let candidate = if profile == "debug" {
        project_dir
            .join("addons")
            .join("grust_pcg")
            .join("pcg")
            .join("graph_editor")
            .join("target")
            .join(profile)
            .join(binary_name)
    } else {
        project_dir
            .join("addons")
            .join("grust_pcg")
            .join(os)
            .join(binary_name)
    };
    godot_print!("{:?}", candidate.to_str().unwrap());

    candidate.exists().then_some(candidate)
}

fn open_graph_editor() {
    godot_print!("OPENING GRAPH EDITOR");

    let project_dir = PathBuf::from(
        ProjectSettings::singleton()
            .globalize_path("res://")
            .to_string(),
    );
    godot_print!("Project dir: {:?}", project_dir);

    let workspace_root = project_dir.parent();
    godot_print!("Workspace root: {:?}", workspace_root);

    let Some(path) = find_graph_editor_executable() else {
        godot_error!("Could not locate graph_editor executable");
        return;
    };
    godot_print!("Launching: {:?}", path);
    match std::process::Command::new(&path).spawn() {
        Ok(_) => godot_print!("Launched"),
        Err(e) => godot_error!("Failed spawn: {:?}", e),
    }
}

impl PCGZone {
    fn run(&mut self) {
        for child in self.base_mut().get_children().iter_shared() {
            child.free();
        }
        let real_path = ProjectSettings::singleton()
            .globalize_path(&self.file_path)
            .to_string();
        let Some(graph) = load_graph_file(&real_path) else {
            godot_print!("PCGZone: couldn't load graph at {}", self.file_path);
            return;
        };

        let mut seeds: HashMap<Uuid, PinValue> = HashMap::new();
        let mut meshes: HashMap<Uuid, Gd<Mesh>> = HashMap::new();
        for node in &graph.nodes {
            //Seed stuff in here which needs data from godot.
            if node.node_type == PCGNodeType::GetSplines {
                let spline_array = &self.splines;
                let mut spline_data: Vec<SplineData> = vec![];
                for spline in spline_array.iter_shared() {
                    if let Some(curve) = spline.get_curve() {
                        spline_data.push(path3d_to_spline_data(&curve, &spline));
                    }
                }
                seeds.insert(node.outputs[0].id, PinValue::SplineArray(spline_data));
            } else if node.node_type == PCGNodeType::FloatInput {
                seeds.insert(node.outputs[0].id, PinValue::Float(self.float_input as f32));
            } else if node.node_type == PCGNodeType::MeshInput {
                let smeshes = &self.spawn_meshes;
                let mut mesh_refs: Vec<MeshRef> = vec![];
                for mesh in smeshes.iter_shared() {
                    let mesh_id = Uuid::new_v4();
                    mesh_refs.push(MeshRef {
                        id: mesh_id,
                        count: 0,
                        probability: 1.0,
                    });
                    meshes.insert(mesh_id, mesh.clone()); // Gd<T> is cheap to clone, just a ref-counted handle
                }
                seeds.insert(node.outputs[0].id, PinValue::MeshArray(mesh_refs));
            }
        }

        let mut host = GodotHost { zone: self, meshes };
        if let Err(e) = evaluate(&graph, &seeds, &mut host) {
            godot_print!("PCGZone evaluation failed: {:?}", e);
        }
    }
    fn open_ge(&mut self) {
        open_graph_editor();
    }
    fn snap_points(
        &mut self,
        points: &Vec<PCGPoint>,
        from_height: f32,
        distance: f32,
    ) -> Vec<PCGPoint> {
        godot_print!("Snapp");
        // 1. Get the World3D via self.base()
        let world = self.base().get_world_3d().expect("World3D not found");

        // 2. Get the space RID (required for space_set_active)
        let space_rid = world.get_space();

        // 3. Force Godot to update collider positions in the physics engine
        let mut phys_server = godot::classes::PhysicsServer3D::singleton();
        phys_server.space_set_active(space_rid, true);

        // 4. Get the space state for casting rays
        let mut space_state = world
            .get_direct_space_state()
            .expect("Physics space state not found");

        // 5. Use new_gd() instead of new_alloc() because this is a RefCounted object!
        let mut query = PhysicsRayQueryParameters3D::new_gd();

        let mut snapped_points: Vec<PCGPoint> = vec![];

        for p in points {
            let start = p.position;

            // Update the query in-place
            query.set_from(Vector3 {
                x: start.x,
                y: start.y + from_height,
                z: start.z,
            });
            query.set_to(Vector3 {
                x: start.x,
                y: start.y - distance,
                z: start.z,
            });

            // Run the ray cast
            let result = space_state.intersect_ray(&query);
            godot_print!(
                "ray from {:?} to {:?} -> hit: {}",
                query.get_from(),
                query.get_to(),
                !result.is_empty()
            );

            if !result.is_empty() {
                // Handle your hit data here
                let position = result.get("position").unwrap().to::<Vector3>();
                let normal = result.get("normal").unwrap().to::<Vector3>();
                let quat = Quaternion::from_rotation_arc(Vector3::UP, normal);
                snapped_points.push(PCGPoint {
                    position: vec3a(position.x, position.y, position.z),
                    rotation: Quat::from_xyzw(quat.x, quat.y, quat.z, quat.w),
                    scale: p.scale,
                    ..Default::default()
                });
            }
        }
        snapped_points
    }
}

fn path3d_to_spline_data(curve: &Gd<Curve3D>, path3d: &Gd<Path3D>) -> SplineData {
    let xform = path3d.get_global_transform();
    let mut points = Vec::new();

    for i in 0..curve.get_point_count() {
        let world_pos = xform * curve.get_point_position(i); // full transform: rotate+scale+translate
        let world_in = xform.basis * curve.get_point_in(i); // basis only: rotate+scale, no translate
        let world_out = xform.basis * curve.get_point_out(i);

        points.push(SplinePoint {
            position: vec3a(world_pos.x, world_pos.y, world_pos.z),
            in_handle: vec3a(world_in.x, world_in.y, world_in.z),
            out_handle: vec3a(world_out.x, world_out.y, world_out.z),
        });
    }
    SplineData {
        points,
        closed: curve.is_closed(),
    }
}

impl GraphHost for GodotHost<'_> {
    fn spawn_meshes(&mut self, meshes: &mut Vec<MeshRef>, points: &[PCGPoint]) {
        let mut rng = rng();

        meshes.iter_mut().for_each(|m| m.count = 0);

        let weights: Vec<f32> = meshes.iter().map(|m| m.probability.abs()).collect();
        let dist = WeightedIndex::new(weights).unwrap();

        // FIX 1: Store exactly which mesh each point belongs to
        let mut point_assignments = Vec::with_capacity(points.len());
        for _ in 0..points.len() {
            let mesh_idx = dist.sample(&mut rng);
            meshes[mesh_idx].count += 1;
            point_assignments.push(mesh_idx);
        }

        let mut multi_meshes: Vec<Gd<MultiMesh>> = meshes
            .iter()
            .map(|m| {
                let mut mm = MultiMesh::new_gd();
                mm.set_mesh(self.meshes.get(&m.id));
                mm.set_transform_format(multi_mesh::TransformFormat::TRANSFORM_3D);
                mm.set_instance_count(m.count);
                mm
            })
            .collect();

        let mut spawned_counts: Vec<i32> = vec![0; multi_meshes.len()];

        let inst_global = self.zone.base().get_global_transform();
        let inv = inst_global.affine_inverse();

        for (idx, p) in points.iter().enumerate() {
            let world_pos = Vector3::new(p.position.x, p.position.y, p.position.z);
            let local_pos = inv * world_pos;
            let xform = Transform3D::new(Basis::IDENTITY, local_pos);

            // FIX 2: Retrieve the exact mesh index we decided on earlier
            let i = point_assignments[idx];

            // FIX 3: Get the 0-based index first, apply transform, THEN increment
            let instance_index = spawned_counts[i];
            multi_meshes[i].set_instance_transform(instance_index, xform);
            spawned_counts[i] += 1;
        }

        let owner = self
            .zone
            .base()
            .get_owner()
            .unwrap_or_else(|| self.zone.base().clone().upcast());

        for mm in multi_meshes {
            let mut inst = MultiMeshInstance3D::new_alloc();
            inst.set_multimesh(&mm);
            self.zone.base_mut().add_child(&inst);
            inst.set_owner(&owner);
        }
    }
    fn snap_points_to_surface(
        &mut self,
        points: &Vec<PCGPoint>,
        from_height: f32,
        distance: f32,
    ) -> Vec<PCGPoint> {
        self.zone.snap_points(points, from_height, distance)
    }
    fn get_meshs_from_indexes(&mut self, is: &Vec<usize>) -> Vec<MeshRef> {
        let mut refs: Vec<MeshRef> = vec![];
        for i in is {
            if let Some(m) = self.zone.spawn_meshes.get(i.to_owned()) {
                let id = Uuid::new_v4();
                refs.push(MeshRef {
                    id: id,
                    count: 0,
                    probability: 1.0,
                });
                self.meshes.insert(id, m);
            }
        }
        refs
    }
    fn get_splines_from_indexes(&mut self, is: &Vec<usize>) -> Vec<SplineData> {
        is.iter()
            .filter_map(|i| {
                // Use and_then to cleanly chain optional lookups
                let s = self.zone.splines.get(*i)?;
                let curve = s.get_curve()?;

                // Return the successful data wrapped in Some
                Some(path3d_to_spline_data(&curve, &s)) // Note: fixed `&spline` to `&s` assuming typo
            })
            .collect()
    }
    fn gprint(&mut self, txt: String) {
        godot_print!("{}", txt)
    }
}
