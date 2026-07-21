pub mod mesh_ref {
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct MeshRef {
        pub id: Uuid,
        pub count: i32,
        pub probability: f32,
    }
}
