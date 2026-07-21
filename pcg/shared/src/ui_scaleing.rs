pub mod tf {
    use iced::{
        Point,
        widget::canvas::{self, path::lyon_path::math::Transform},
    };
    pub fn tp(p: Point, trans: (f32, f32, f32, f32)) -> Point {
        Point {
            x: (p.x * trans.2 + trans.0),
            y: (p.y * trans.3 + trans.1),
        }
    }

    pub fn untp(p: Point, trans: (f32, f32, f32, f32)) -> Point {
        Point {
            x: (p.x - trans.0) / trans.2,
            y: (p.y - trans.1) / trans.3,
        }
    }

    pub fn tfs(a: f32, b: f32, trans: (f32, f32, f32, f32)) -> (f32, f32) {
        (a * trans.2, b * trans.3)
    }

    pub fn tf(f: f32, trans: (f32, f32, f32, f32)) -> f32 {
        f * (trans.2.powi(2) + trans.3.powi(2)).sqrt()
    }

    pub fn transform_path(pth: canvas::Path, trans: (f32, f32, f32, f32)) -> canvas::Path {
        pth.transform(&Transform::scale(trans.2, trans.3))
            .transform(&Transform::translation(trans.0, trans.1))
    }
}
