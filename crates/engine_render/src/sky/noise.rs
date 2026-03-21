/// WGSL noise function library.
/// Ported from Keijiro Takahashi's NoiseShader (MIT), extended with Worley, FBM, and curl noise.
/// All functions are `const &str` WGSL snippets concatenated into consuming shaders.

pub const NOISE_COMMON_WGSL: &str = r#"
fn wgl_mod289_f(x: f32) -> f32 { return x - floor(x / 289.0) * 289.0; }
fn wgl_mod289_v2(x: vec2<f32>) -> vec2<f32> { return x - floor(x / 289.0) * 289.0; }
fn wgl_mod289_v3(x: vec3<f32>) -> vec3<f32> { return x - floor(x / 289.0) * 289.0; }
fn wgl_mod289_v4(x: vec4<f32>) -> vec4<f32> { return x - floor(x / 289.0) * 289.0; }

fn wgl_permute_v2(x: vec2<f32>) -> vec2<f32> { return wgl_mod289_v2((x * 34.0 + 10.0) * x); }
fn wgl_permute_v3(x: vec3<f32>) -> vec3<f32> { return wgl_mod289_v3((x * 34.0 + 10.0) * x); }
fn wgl_permute_v4(x: vec4<f32>) -> vec4<f32> { return wgl_mod289_v4((x * 34.0 + 10.0) * x); }

fn wgl_fade_v2(t: vec2<f32>) -> vec2<f32> { return t * t * t * (t * (t * 6.0 - 15.0) + 10.0); }
fn wgl_fade_v3(t: vec3<f32>) -> vec3<f32> { return t * t * t * (t * (t * 6.0 - 15.0) + 10.0); }
"#;

pub const CLASSIC_NOISE_2D_WGSL: &str = r#"
fn classic_noise_2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let pi0 = wgl_mod289_v2(i);
    let pi1 = wgl_mod289_v2(i + 1.0);

    let ix = vec4<f32>(pi0.x, pi1.x, pi0.x, pi1.x);
    let iy = vec4<f32>(pi0.y, pi0.y, pi1.y, pi1.y);
    let fx = vec4<f32>(f.x, f.x - 1.0, f.x, f.x - 1.0);
    let fy = vec4<f32>(f.y, f.y, f.y - 1.0, f.y - 1.0);

    let ip = wgl_permute_v4(wgl_permute_v4(ix) + iy);

    let phi = ip / 41.0 * 6.28318530718;
    let g00 = vec2<f32>(cos(phi.x), sin(phi.x));
    let g10 = vec2<f32>(cos(phi.y), sin(phi.y));
    let g01 = vec2<f32>(cos(phi.z), sin(phi.z));
    let g11 = vec2<f32>(cos(phi.w), sin(phi.w));

    let n00 = dot(g00, vec2<f32>(fx.x, fy.x));
    let n10 = dot(g10, vec2<f32>(fx.y, fy.y));
    let n01 = dot(g01, vec2<f32>(fx.z, fy.z));
    let n11 = dot(g11, vec2<f32>(fx.w, fy.w));

    let fade_xy = wgl_fade_v2(f);
    let n_x = mix(vec2<f32>(n00, n01), vec2<f32>(n10, n11), fade_xy.x);
    let n_xy = mix(n_x.x, n_x.y, fade_xy.y);
    return 1.41421356 * n_xy;
}
"#;

pub const CLASSIC_NOISE_3D_WGSL: &str = r#"
fn classic_noise_3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let pi0 = wgl_mod289_v3(i);
    let pi1 = wgl_mod289_v3(i + 1.0);

    let ix = vec4<f32>(pi0.x, pi1.x, pi0.x, pi1.x);
    let iy = vec4<f32>(pi0.y, pi0.y, pi1.y, pi1.y);

    let ixy = wgl_permute_v4(wgl_permute_v4(ix) + iy);
    let ixy0 = wgl_permute_v4(ixy + vec4<f32>(pi0.z));
    let ixy1 = wgl_permute_v4(ixy + vec4<f32>(pi1.z));

    var gx0 = ixy0 / 7.0; gx0 = fract(gx0) * 2.0 - 1.0;
    var gy0 = floor(ixy0 / 7.0) / 7.0; gy0 = fract(gy0) * 2.0 - 1.0;
    let gz0 = 1.0 - abs(gx0) - abs(gy0);

    let s0x = step(vec4<f32>(0.0), -gx0);
    let s0y = step(vec4<f32>(0.0), -gy0);
    gx0 = gx0 + s0x * select(vec4<f32>(-1.0), vec4<f32>(1.0), gz0 < vec4<f32>(0.0));
    gy0 = gy0 + s0y * select(vec4<f32>(-1.0), vec4<f32>(1.0), gz0 < vec4<f32>(0.0));

    var gx1 = ixy1 / 7.0; gx1 = fract(gx1) * 2.0 - 1.0;
    var gy1 = floor(ixy1 / 7.0) / 7.0; gy1 = fract(gy1) * 2.0 - 1.0;
    let gz1 = 1.0 - abs(gx1) - abs(gy1);

    let s1x = step(vec4<f32>(0.0), -gx1);
    let s1y = step(vec4<f32>(0.0), -gy1);
    gx1 = gx1 + s1x * select(vec4<f32>(-1.0), vec4<f32>(1.0), gz1 < vec4<f32>(0.0));
    gy1 = gy1 + s1y * select(vec4<f32>(-1.0), vec4<f32>(1.0), gz1 < vec4<f32>(0.0));

    let g000 = normalize(vec3<f32>(gx0.x, gy0.x, gz0.x));
    let g100 = normalize(vec3<f32>(gx0.y, gy0.y, gz0.y));
    let g010 = normalize(vec3<f32>(gx0.z, gy0.z, gz0.z));
    let g110 = normalize(vec3<f32>(gx0.w, gy0.w, gz0.w));
    let g001 = normalize(vec3<f32>(gx1.x, gy1.x, gz1.x));
    let g101 = normalize(vec3<f32>(gx1.y, gy1.y, gz1.y));
    let g011 = normalize(vec3<f32>(gx1.z, gy1.z, gz1.z));
    let g111 = normalize(vec3<f32>(gx1.w, gy1.w, gz1.w));

    let pf0 = f;
    let pf1 = f - 1.0;

    let n000 = dot(g000, pf0);
    let n100 = dot(g100, vec3<f32>(pf1.x, pf0.y, pf0.z));
    let n010 = dot(g010, vec3<f32>(pf0.x, pf1.y, pf0.z));
    let n110 = dot(g110, vec3<f32>(pf1.x, pf1.y, pf0.z));
    let n001 = dot(g001, vec3<f32>(pf0.x, pf0.y, pf1.z));
    let n101 = dot(g101, vec3<f32>(pf1.x, pf0.y, pf1.z));
    let n011 = dot(g011, vec3<f32>(pf0.x, pf1.y, pf1.z));
    let n111 = dot(g111, pf1);

    let fade_xyz = wgl_fade_v3(pf0);
    let n_z = mix(vec4<f32>(n000, n100, n010, n110),
                  vec4<f32>(n001, n101, n011, n111), fade_xyz.z);
    let n_yz = mix(n_z.xy, n_z.zw, fade_xyz.y);
    let n_xyz = mix(n_yz.x, n_yz.y, fade_xyz.x);
    return 1.4 * n_xyz;
}
"#;

pub const SIMPLEX_NOISE_3D_WGSL: &str = r#"
fn simplex_noise_3d(v: vec3<f32>) -> f32 {
    let C = vec2<f32>(1.0 / 6.0, 1.0 / 3.0);
    let i = floor(v + dot(v, vec3<f32>(C.y)));
    let x0 = v - i + dot(i, vec3<f32>(C.x));

    let g = step(x0.yzx, x0.xyz);
    let l = 1.0 - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    let x1 = x0 - i1 + C.x;
    let x2 = x0 - i2 + C.y;
    let x3 = x0 - 0.5;

    let ii = wgl_mod289_v3(i);
    let p0 = wgl_permute_v4(vec4<f32>(ii.z, ii.z + i1.z, ii.z + i2.z, ii.z + 1.0));
    let p1 = wgl_permute_v4(p0 + vec4<f32>(ii.y, ii.y + i1.y, ii.y + i2.y, ii.y + 1.0));
    let p  = wgl_permute_v4(p1 + vec4<f32>(ii.x, ii.x + i1.x, ii.x + i2.x, ii.x + 1.0));

    var gx = fract(p / 7.0) * 2.0 - 1.0;
    var gy = fract(floor(p / 7.0) / 7.0) * 2.0 - 1.0;
    let gz = 1.0 - abs(gx) - abs(gy);

    let sx = step(vec4<f32>(0.0), -gx);
    let sy = step(vec4<f32>(0.0), -gy);
    gx = gx + sx * select(vec4<f32>(-1.0), vec4<f32>(1.0), gz < vec4<f32>(0.0));
    gy = gy + sy * select(vec4<f32>(-1.0), vec4<f32>(1.0), gz < vec4<f32>(0.0));

    let g0 = normalize(vec3<f32>(gx.x, gy.x, gz.x));
    let g1 = normalize(vec3<f32>(gx.y, gy.y, gz.y));
    let g2 = normalize(vec3<f32>(gx.z, gy.z, gz.z));
    let g3 = normalize(vec3<f32>(gx.w, gy.w, gz.w));

    var m = vec4<f32>(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
    m = max(0.5 - m, vec4<f32>(0.0));
    let m3 = m * m * m;
    let m4 = m * m3;

    let px = vec4<f32>(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
    return 107.0 * dot(m4, px);
}
"#;

pub const WORLEY_NOISE_3D_WGSL: &str = r#"
fn hash_3d(p: vec3<f32>) -> vec3<f32> {
    var q = vec3<f32>(
        dot(p, vec3<f32>(127.1, 311.7, 74.7)),
        dot(p, vec3<f32>(269.5, 183.3, 246.1)),
        dot(p, vec3<f32>(113.5, 271.9, 124.6)),
    );
    return fract(sin(q) * 43758.5453123);
}

fn worley_noise_3d(p: vec3<f32>) -> f32 {
    let cell = floor(p);
    let local = fract(p);
    var min_dist = 1.0;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            for (var z = -1; z <= 1; z++) {
                let offset = vec3<f32>(f32(x), f32(y), f32(z));
                let feature = hash_3d(cell + offset);
                let diff = offset + feature - local;
                let d = dot(diff, diff);
                min_dist = min(min_dist, d);
            }
        }
    }
    return sqrt(min_dist);
}
"#;

pub const FBM_WGSL: &str = r#"
fn fbm_classic_3d(p: vec3<f32>, octaves: i32) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    var pos = p;
    for (var i = 0; i < octaves; i++) {
        val += amp * classic_noise_3d(pos * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    return val;
}

fn fbm_simplex_3d(p: vec3<f32>, octaves: i32) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    var pos = p;
    for (var i = 0; i < octaves; i++) {
        val += amp * simplex_noise_3d(pos * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    return val;
}

fn fbm_classic_2d(p: vec2<f32>, octaves: i32) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    for (var i = 0; i < octaves; i++) {
        val += amp * classic_noise_2d(p * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    return val;
}
"#;

pub const CURL_NOISE_WGSL: &str = r#"
fn curl_noise_3d(p: vec3<f32>) -> vec3<f32> {
    let e = 0.001;
    let dx = vec3<f32>(e, 0.0, 0.0);
    let dy = vec3<f32>(0.0, e, 0.0);
    let dz = vec3<f32>(0.0, 0.0, e);

    let px = simplex_noise_3d(p + dx) - simplex_noise_3d(p - dx);
    let py = simplex_noise_3d(p + dy) - simplex_noise_3d(p - dy);
    let pz = simplex_noise_3d(p + dz) - simplex_noise_3d(p - dz);

    let curl_x = (simplex_noise_3d(p + dy + vec3<f32>(31.41, 0.0, 0.0)) -
                  simplex_noise_3d(p - dy + vec3<f32>(31.41, 0.0, 0.0))) -
                 (simplex_noise_3d(p + dz + vec3<f32>(0.0, 27.18, 0.0)) -
                  simplex_noise_3d(p - dz + vec3<f32>(0.0, 27.18, 0.0)));
    let curl_y = (simplex_noise_3d(p + dz + vec3<f32>(0.0, 0.0, 14.14)) -
                  simplex_noise_3d(p - dz + vec3<f32>(0.0, 0.0, 14.14))) -
                 (simplex_noise_3d(p + dx + vec3<f32>(17.32, 0.0, 0.0)) -
                  simplex_noise_3d(p - dx + vec3<f32>(17.32, 0.0, 0.0)));
    let curl_z = (simplex_noise_3d(p + dx + vec3<f32>(0.0, 22.36, 0.0)) -
                  simplex_noise_3d(p - dx + vec3<f32>(0.0, 22.36, 0.0))) -
                 (simplex_noise_3d(p + dy + vec3<f32>(0.0, 0.0, 19.42)) -
                  simplex_noise_3d(p - dy + vec3<f32>(0.0, 0.0, 19.42)));

    return vec3<f32>(curl_x, curl_y, curl_z) / (2.0 * e);
}
"#;

/// All noise functions concatenated for inclusion in compute/fragment shaders.
pub fn all_noise_wgsl() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        NOISE_COMMON_WGSL,
        CLASSIC_NOISE_2D_WGSL,
        CLASSIC_NOISE_3D_WGSL,
        SIMPLEX_NOISE_3D_WGSL,
        WORLEY_NOISE_3D_WGSL,
        FBM_WGSL,
        CURL_NOISE_WGSL,
    )
}
