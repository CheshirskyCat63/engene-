//! Phase 3.2: 3D cloud noise textures.
//! Perlin-Worley 128^3 RGBA and Worley 32^3 R8Unorm, CPU-generated and uploaded.

const PERLIN_WORLEY_SIZE: u32 = 128;
const WORLEY_SIZE: u32 = 32;

/// Perlin-Worley (128^3 RGBA) and Worley (32^3 R8Unorm) 3D cloud noise textures.
pub struct CloudNoiseTextures {
    pub perlin_worley_3d: wgpu::Texture,
    pub perlin_worley_view: wgpu::TextureView,
    pub worley_3d: wgpu::Texture,
    pub worley_view: wgpu::TextureView,
}

/// Deterministic hash for gradient noise.
#[inline]
fn hash3(p: [i32; 3]) -> [f32; 3] {
    let n = p[0]
        .wrapping_add(p[1].wrapping_mul(57))
        .wrapping_add(p[2].wrapping_mul(113));
    let f = ((n & 0x7fff_ffff) as f32) / 2147483647.0;
    let theta = f * 6.2831853;
    let phi = ((n.wrapping_mul(31) & 0x7fff_ffff) as f32 / 2147483647.0) * 3.1415926;
    [
        theta.sin() * phi.cos(),
        theta.sin() * phi.sin(),
        phi.cos(),
    ]
}

/// Value/gradient noise 3D (deterministic, Perlin-like).
fn gradient_noise_3d(p: [f32; 3]) -> f32 {
    let i0 = [
        p[0].floor() as i32,
        p[1].floor() as i32,
        p[2].floor() as i32,
    ];
    let f = [
        p[0] - i0[0] as f32,
        p[1] - i0[1] as f32,
        p[2] - i0[2] as f32,
    ];
    let u = f[0] * f[0] * (3.0 - 2.0 * f[0]);
    let v = f[1] * f[1] * (3.0 - 2.0 * f[1]);
    let w = f[2] * f[2] * (3.0 - 2.0 * f[2]);

    let mut n = 0.0f32;
    for dx in 0..2 {
        for dy in 0..2 {
            for dz in 0..2 {
                let gi = [i0[0] + dx, i0[1] + dy, i0[2] + dz];
                let g = hash3(gi);
                let diff = [
                    dx as f32 - f[0],
                    dy as f32 - f[1],
                    dz as f32 - f[2],
                ];
                let t = diff[0] * g[0] + diff[1] * g[1] + diff[2] * g[2];
                let wx = if dx == 0 { 1.0 - u } else { u };
                let wy = if dy == 0 { 1.0 - v } else { v };
                let wz = if dz == 0 { 1.0 - w } else { w };
                n += t * wx * wy * wz;
            }
        }
    }
    (n * 0.5 + 0.5).clamp(0.0, 1.0)
}

/// Worley/cellular 3D: returns minimum distance to cell center.
fn worley_noise_3d(p: [f32; 3]) -> f32 {
    let cell = [
        p[0].floor() as i32,
        p[1].floor() as i32,
        p[2].floor() as i32,
    ];
    let local = [
        p[0] - cell[0] as f32,
        p[1] - cell[1] as f32,
        p[2] - cell[2] as f32,
    ];

    let mut min_dist = 1.0f32;
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let nc = [cell[0] + x, cell[1] + y, cell[2] + z];
                let seed = nc[0]
                    .wrapping_add(nc[1].wrapping_mul(57))
                    .wrapping_add(nc[2].wrapping_mul(113));
                let hx = ((seed & 0x7fff) as f32) / 32768.0;
                let hy = (((seed.wrapping_mul(31)) & 0x7fff) as f32) / 32768.0;
                let hz = (((seed.wrapping_mul(47)) & 0x7fff) as f32) / 32768.0;
                let dx = x as f32 + hx - local[0];
                let dy = y as f32 + hy - local[1];
                let dz = z as f32 + hz - local[2];
                let d = dx * dx + dy * dy + dz * dz;
                min_dist = min_dist.min(d);
            }
        }
    }
    min_dist.sqrt().min(1.0)
}

/// Generate Perlin-Worley 128^3 RGBA (Rgba8Unorm).
fn generate_perlin_worley_3d() -> Vec<u8> {
    let size = PERLIN_WORLEY_SIZE as usize;
    let mut out = vec![0u8; size * size * size * 4];

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let scale = 0.02;
                let p = [
                    x as f32 * scale,
                    y as f32 * scale,
                    z as f32 * scale,
                ];

                let perlin_low = gradient_noise_3d(p);
                let perlin_mid = gradient_noise_3d([p[0] * 2.0, p[1] * 2.0, p[2] * 2.0]);
                let perlin_high = gradient_noise_3d([p[0] * 4.0, p[1] * 4.0, p[2] * 4.0]);
                let worley_val = worley_noise_3d(p);

                let pw_low = ((1.0 - worley_val) * perlin_low).clamp(0.0, 1.0);
                let pw_mid = ((1.0 - worley_val) * perlin_mid).clamp(0.0, 1.0);
                let pw_high = ((1.0 - worley_val) * perlin_high).clamp(0.0, 1.0);
                let w = worley_val.clamp(0.0, 1.0);

                let idx = (z * size * size + y * size + x) * 4;
                out[idx] = (pw_low * 255.0) as u8;
                out[idx + 1] = (pw_mid * 255.0) as u8;
                out[idx + 2] = (pw_high * 255.0) as u8;
                out[idx + 3] = (w * 255.0) as u8;
            }
        }
    }
    out
}

/// Generate Worley 32^3 R8Unorm.
fn generate_worley_3d() -> Vec<u8> {
    let size = WORLEY_SIZE as usize;
    let mut out = vec![0u8; size * size * size];

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let scale = 0.15;
                let p = [
                    x as f32 * scale,
                    y as f32 * scale,
                    z as f32 * scale,
                ];
                let w = worley_noise_3d(p).clamp(0.0, 1.0);
                out[z * size * size + y * size + x] = (w * 255.0) as u8;
            }
        }
    }
    out
}

impl CloudNoiseTextures {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let perlin_worley_data = generate_perlin_worley_3d();
        let worley_data = generate_worley_3d();

        let perlin_worley_3d = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud_perlin_worley_3d"),
            size: wgpu::Extent3d {
                width: PERLIN_WORLEY_SIZE,
                height: PERLIN_WORLEY_SIZE,
                depth_or_array_layers: PERLIN_WORLEY_SIZE,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let worley_3d = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cloud_worley_3d"),
            size: wgpu::Extent3d {
                width: WORLEY_SIZE,
                height: WORLEY_SIZE,
                depth_or_array_layers: WORLEY_SIZE,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &perlin_worley_3d,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &perlin_worley_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(PERLIN_WORLEY_SIZE * 4),
                rows_per_image: Some(PERLIN_WORLEY_SIZE),
            },
            wgpu::Extent3d {
                width: PERLIN_WORLEY_SIZE,
                height: PERLIN_WORLEY_SIZE,
                depth_or_array_layers: PERLIN_WORLEY_SIZE,
            },
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &worley_3d,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &worley_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WORLEY_SIZE),
                rows_per_image: Some(WORLEY_SIZE),
            },
            wgpu::Extent3d {
                width: WORLEY_SIZE,
                height: WORLEY_SIZE,
                depth_or_array_layers: WORLEY_SIZE,
            },
        );

        let perlin_worley_view = perlin_worley_3d.create_view(&Default::default());
        let worley_view = worley_3d.create_view(&Default::default());

        Self {
            perlin_worley_3d,
            perlin_worley_view,
            worley_3d,
            worley_view,
        }
    }
}
