pub mod point {

    use glam::{Affine3A, Quat, Vec3A};

    #[derive(Debug, Clone, Copy)]
    pub struct PCGPoint {
        pub position: Vec3A,
        pub rotation: Quat,
        pub scale: Vec3A,
        pub density: f32,
        pub distance: f32,
    }

    impl PCGPoint {
        pub const IDENTITY: Self = Self {
            position: Vec3A::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3A::ONE,
            density: 1.0,
            distance: 0.0,
        };

        pub fn from_position(position: Vec3A) -> Self {
            Self {
                position,
                ..Self::IDENTITY
            }
        }

        pub fn to_affine(&self) -> Affine3A {
            Affine3A::from_scale_rotation_translation(
                self.scale.into(),
                self.rotation,
                self.position.into(),
            )
        }
    }
    impl Default for PCGPoint {
        fn default() -> Self {
            Self::IDENTITY
        }
    }
}
