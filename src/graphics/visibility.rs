use glam::{Mat4, Vec3, Vec4};

pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    pub fn from_view_projection(vp: &Mat4) -> Self {
        let c = vp.to_cols_array_2d();
        let row = |r: usize| -> [f32; 4] {
            [c[0][r], c[1][r], c[2][r], c[3][r]]
        };
        let r0 = row(0);
        let r1 = row(1);
        let r2 = row(2);
        let r3 = row(3);

        let mut planes = [Vec4::ZERO; 6];
        // left
        planes[0] = Vec4::new(r3[0]+r0[0], r3[1]+r0[1], r3[2]+r0[2], r3[3]+r0[3]);
        // right
        planes[1] = Vec4::new(r3[0]-r0[0], r3[1]-r0[1], r3[2]-r0[2], r3[3]-r0[3]);
        // bottom
        planes[2] = Vec4::new(r3[0]+r1[0], r3[1]+r1[1], r3[2]+r1[2], r3[3]+r1[3]);
        // top
        planes[3] = Vec4::new(r3[0]-r1[0], r3[1]-r1[1], r3[2]-r1[2], r3[3]-r1[3]);
        // near
        planes[4] = Vec4::new(r2[0], r2[1], r2[2], r2[3]);
        // far
        planes[5] = Vec4::new(r3[0]-r2[0], r3[1]-r2[1], r3[2]-r2[2], r3[3]-r2[3]);

        for p in &mut planes {
            let len = Vec3::new(p.x, p.y, p.z).length();
            if len > 1e-6 {
                *p /= len;
            }
        }

        Self { planes }
    }

    pub fn test_sphere(&self, center: Vec3, radius: f32) -> bool {
        for p in &self.planes {
            let dist = p.x * center.x + p.y * center.y + p.z * center.z + p.w;
            if dist < -radius {
                return false;
            }
        }
        true
    }
}
