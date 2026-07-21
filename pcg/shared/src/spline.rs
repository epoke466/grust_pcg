pub mod spline {
    use crate::PCGPoint;
    use glam::{Quat, Vec3A, vec3a};

    #[derive(Debug, Clone)]
    pub struct SplineData {
        pub points: Vec<SplinePoint>,
        pub closed: bool,
    }

    #[derive(Debug, Clone)]
    pub struct SplinePoint {
        pub position: Vec3A,
        pub in_handle: Vec3A,
        pub out_handle: Vec3A,
    }

    fn cubic_bezier(p0: Vec3A, p1: Vec3A, p2: Vec3A, p3: Vec3A, t: f32) -> Vec3A {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;
        let (a, b, c, d) = (mt2 * mt, 3.0 * mt2 * t, 3.0 * mt * t2, t2 * t);
        p0 * a + p1 * b + p2 * c + p3 * d
    }

    pub fn sample_spline(spline: &SplineData, density: f32) -> Vec<PCGPoint> {
        let n = spline.points.len();
        if n < 2 {
            return spline
                .points
                .first()
                .map(|p| vec![PCGPoint::from_position(p.position)])
                .unwrap_or_default();
        }
        if density <= 0.0 {
            return Vec::new();
        }

        let samples_per_segment = density.max(1.0) as usize;
        let segment_count = if spline.closed { n } else { n - 1 };
        let mut out = Vec::new();

        for seg in 0..segment_count {
            let p0 = &spline.points[seg];
            let p1 = &spline.points[(seg + 1) % n];
            let c0 = p0.position;
            let c1 = p0.position + p0.out_handle;
            let c2 = p1.position + p1.in_handle;
            let c3 = p1.position;

            for s in 0..samples_per_segment {
                let t = s as f32 / samples_per_segment as f32;
                let pos = cubic_bezier(c0, c1, c2, c3, t);
                out.push(PCGPoint::from_position(pos));
            }
        }
        if !spline.closed {
            out.push(PCGPoint::from_position(spline.points[n - 1].position));
        }
        out
    }

    fn point_in_polygon_xz(x: f32, z: f32, polygon: &[PCGPoint]) -> bool {
        let n = polygon.len();
        if n < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let xi = polygon[i].position.x;
            let zi = polygon[i].position.z;
            let xj = polygon[j].position.x;
            let zj = polygon[j].position.z;
            if (zi > z) != (zj > z) && x < (xj - xi) * (z - zi) / (zj - zi) + xi {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    pub fn point_grid_xz_from_points(points: &[PCGPoint], spacing: f32) -> Vec<PCGPoint> {
        if spacing <= 0.0 || points.len() < 3 {
            return Vec::new();
        }

        let mut low = vec3a(f32::MAX, f32::MAX, f32::MAX);
        let mut high = vec3a(f32::MIN, f32::MIN, f32::MIN);
        for point in points {
            low.x = low.x.min(point.position.x);
            low.y = low.y.min(point.position.y);
            low.z = low.z.min(point.position.z);
            high.x = high.x.max(point.position.x);
            high.y = high.y.max(point.position.y);
            high.z = high.z.max(point.position.z);
        }

        let y = (low.y + high.y) / 2.0;
        let mut point_grid: Vec<PCGPoint> = vec![];

        let mut x = low.x;
        while x < high.x {
            let mut z = low.z;
            while z < high.z {
                if point_in_polygon_xz(x, z, points) {
                    point_grid.push(PCGPoint {
                        position: vec3a(x, y, z),
                        rotation: Quat::default(),
                        scale: vec3a(1.0, 1.0, 1.0),
                        ..Default::default()
                    });
                }
                z += spacing;
            }
            x += spacing;
        }

        point_grid
    }
}
