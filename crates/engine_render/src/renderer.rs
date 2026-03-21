use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::window::Window;

use crate::graphics::ibl::IblBindings;
use crate::graphics::mesh::{CapsuleMesh, EntityInstance, MeshVertex};
use crate::graphics::pbr::{PbrBindings, SunLight};
use crate::graphics::postprocess::{BloomPass, HdrTarget, ToneMapPass};
use crate::graphics::shadow::{CascadedShadowMap, ShadowUniforms, CASCADE_COUNT};
use crate::graphics::skinning::{SkinBuffer, SkinVertex};
use crate::graphics::skybox::SkyboxPass;
use crate::graphics::taa::TaaPass;
use crate::graphics::terrain::{TerrainMesh, TerrainVertex};
use crate::graphics::vegetation::VegetationSystem;
use crate::world::biome::Biome;
use crate::world::heightmap::Heightmap;

// ---------------------------------------------------------------------------
// Shaders
// ---------------------------------------------------------------------------

const PBR_TERRAIN_SHADER: &str = r#"
struct Camera { view_proj: mat4x4<f32> };
struct Light  { direction: vec4<f32>, color: vec4<f32>, ambient: vec4<f32>, camera_pos: vec4<f32> };
struct Shadow { light_vp: array<mat4x4<f32>, 4>, splits: vec4<f32> };

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> light: Light;
@group(2) @binding(0) var shadow_tex: texture_depth_2d_array;
@group(2) @binding(1) var shadow_samp: sampler_comparison;
@group(2) @binding(2) var<uniform> shadow: Shadow;

struct VsIn  { @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) color: vec3<f32> };
struct VsOut { @builtin(position) clip_pos: vec4<f32>, @location(0) world_pos: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) color: vec3<f32> };

@vertex fn vs_main(in: VsIn) -> VsOut {
    var o: VsOut;
    o.clip_pos  = camera.view_proj * vec4<f32>(in.position, 1.0);
    o.world_pos = in.position;
    o.normal    = in.normal;
    o.color     = in.color;
    return o;
}

const PI: f32 = 3.14159265359;
fn ggx_d(nh: f32, r: f32) -> f32 { let a=r*r; let a2=a*a; let d=nh*nh*(a2-1.0)+1.0; return a2/(PI*d*d+1e-4); }
fn ggx_g1(nv: f32, r: f32) -> f32 { let k=((r+1.0)*(r+1.0))/8.0; return nv/(nv*(1.0-k)+k+1e-4); }
fn ggx_g(nv: f32, nl: f32, r: f32) -> f32 { return ggx_g1(nv,r)*ggx_g1(nl,r); }
fn fresnel(ct: f32, f0: vec3<f32>) -> vec3<f32> { return f0+(1.0-f0)*pow(clamp(1.0-ct,0.0,1.0),5.0); }

fn calc_shadow(wp: vec3<f32>) -> f32 {
    let d = length(wp - light.camera_pos.xyz);
    var ci: u32 = 3u;
    if d < shadow.splits.x { ci = 0u; }
    else if d < shadow.splits.y { ci = 1u; }
    else if d < shadow.splits.z { ci = 2u; }
    let lp = shadow.light_vp[ci] * vec4<f32>(wp, 1.0);
    let ndc = lp.xyz / lp.w;
    let uv = ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    if uv.x<0.0 || uv.x>1.0 || uv.y<0.0 || uv.y>1.0 || ndc.z>1.0 || ndc.z<0.0 { return 1.0; }
    return textureSampleCompare(shadow_tex, shadow_samp, uv, ci, ndc.z);
}

@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let albedo = in.color;
    let rough = 0.85; let metal = 0.0;
    let n = normalize(in.normal);
    let v = normalize(light.camera_pos.xyz - in.world_pos);
    let l = normalize(light.direction.xyz);
    let h = normalize(v + l);
    let nl = max(dot(n,l),0.0); let nv = max(dot(n,v),0.001);
    let nh = max(dot(n,h),0.0); let hv = max(dot(h,v),0.0);
    let f0 = mix(vec3<f32>(0.04), albedo, metal);
    let spec = (ggx_d(nh,rough)*ggx_g(nv,nl,rough)*fresnel(hv,f0)) / (4.0*nv*nl+1e-4);
    let kd = (vec3<f32>(1.0)-fresnel(hv,f0))*(1.0-metal);
    let shad = calc_shadow(in.world_pos);
    let lo = (kd*albedo/PI + spec) * light.color.xyz * nl * shad;
    let amb = light.ambient.xyz * albedo;
    return vec4<f32>(amb + lo, 1.0);
}
"#;

const PBR_ENTITY_SHADER: &str = r#"
struct Camera { view_proj: mat4x4<f32> };
struct Light  { direction: vec4<f32>, color: vec4<f32>, ambient: vec4<f32>, camera_pos: vec4<f32> };
struct Shadow { light_vp: array<mat4x4<f32>, 4>, splits: vec4<f32> };

@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> light: Light;
@group(2) @binding(0) var shadow_tex: texture_depth_2d_array;
@group(2) @binding(1) var shadow_samp: sampler_comparison;
@group(2) @binding(2) var<uniform> shadow: Shadow;

struct VertIn { @location(0) position: vec3<f32>, @location(1) normal: vec3<f32> };
struct InstIn { @location(2) world_pos: vec3<f32>, @location(3) color: vec3<f32> };
struct VsOut  { @builtin(position) clip_pos: vec4<f32>, @location(0) world_pos: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) color: vec3<f32> };

@vertex fn vs_main(v: VertIn, i: InstIn) -> VsOut {
    let wp = v.position + i.world_pos;
    var o: VsOut;
    o.clip_pos  = camera.view_proj * vec4<f32>(wp, 1.0);
    o.world_pos = wp;
    o.normal    = v.normal;
    o.color     = i.color;
    return o;
}

const PI: f32 = 3.14159265359;
fn ggx_d(nh: f32, r: f32) -> f32 { let a=r*r; let a2=a*a; let d=nh*nh*(a2-1.0)+1.0; return a2/(PI*d*d+1e-4); }
fn ggx_g1(nv: f32, r: f32) -> f32 { let k=((r+1.0)*(r+1.0))/8.0; return nv/(nv*(1.0-k)+k+1e-4); }
fn ggx_g(nv: f32, nl: f32, r: f32) -> f32 { return ggx_g1(nv,r)*ggx_g1(nl,r); }
fn fresnel(ct: f32, f0: vec3<f32>) -> vec3<f32> { return f0+(1.0-f0)*pow(clamp(1.0-ct,0.0,1.0),5.0); }

fn calc_shadow(wp: vec3<f32>) -> f32 {
    let d = length(wp - light.camera_pos.xyz);
    var ci: u32 = 3u;
    if d < shadow.splits.x { ci = 0u; }
    else if d < shadow.splits.y { ci = 1u; }
    else if d < shadow.splits.z { ci = 2u; }
    let lp = shadow.light_vp[ci] * vec4<f32>(wp, 1.0);
    let ndc = lp.xyz / lp.w;
    let uv = ndc.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    if uv.x<0.0 || uv.x>1.0 || uv.y<0.0 || uv.y>1.0 || ndc.z>1.0 || ndc.z<0.0 { return 1.0; }
    return textureSampleCompare(shadow_tex, shadow_samp, uv, ci, ndc.z);
}

@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let albedo = in.color;
    let rough = 0.6; let metal = 0.0;
    let n = normalize(in.normal);
    let v = normalize(light.camera_pos.xyz - in.world_pos);
    let l = normalize(light.direction.xyz);
    let h = normalize(v + l);
    let nl = max(dot(n,l),0.0); let nv = max(dot(n,v),0.001);
    let nh = max(dot(n,h),0.0); let hv = max(dot(h,v),0.0);
    let f0 = mix(vec3<f32>(0.04), albedo, metal);
    let spec = (ggx_d(nh,rough)*ggx_g(nv,nl,rough)*fresnel(hv,f0)) / (4.0*nv*nl+1e-4);
    let kd = (vec3<f32>(1.0)-fresnel(hv,f0))*(1.0-metal);
    let shad = calc_shadow(in.world_pos);
    let lo = (kd*albedo/PI + spec) * light.color.xyz * nl * shad;
    let amb = light.ambient.xyz * albedo;
    return vec4<f32>(amb + lo, 1.0);
}
"#;

const LINE_SHADER: &str = "
struct Camera { view_proj: mat4x4<f32> };
@group(0) @binding(0) var<uniform> camera: Camera;
struct VsIn  { @location(0) position: vec3<f32>, @location(1) color: vec3<f32> };
struct VsOut { @builtin(position) clip_position: vec4<f32>, @location(0) color: vec3<f32> };
@vertex fn vs_main(in: VsIn) -> VsOut {
    var o: VsOut; o.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0); o.color = in.color; return o;
}
@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> { return vec4<f32>(in.color, 1.0); }
";

const SKINNED_SHADER: &str = "
struct Camera { view_proj: mat4x4<f32> };
struct JointBlock { matrices: array<mat4x4<f32>, 64> };
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var<uniform> joints: JointBlock;
struct VsIn { @location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) joints_idx: vec4<u32>, @location(3) weights: vec4<f32> };
struct VsOut { @builtin(position) clip_position: vec4<f32>, @location(0) normal: vec3<f32> };
@vertex fn vs_main(in: VsIn) -> VsOut {
    let w = in.weights;
    var sm = joints.matrices[in.joints_idx.x]*w.x + joints.matrices[in.joints_idx.y]*w.y
           + joints.matrices[in.joints_idx.z]*w.z + joints.matrices[in.joints_idx.w]*w.w;
    let wp = sm * vec4<f32>(in.position, 1.0);
    let wn = (sm * vec4<f32>(in.normal, 0.0)).xyz;
    var o: VsOut; o.clip_position = camera.view_proj * wp; o.normal = wn; return o;
}
@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let sun = normalize(vec3<f32>(0.3,0.8,0.5)); let n = normalize(in.normal);
    let l = 0.35 + 0.65*max(dot(n,sun),0.0);
    return vec4<f32>(vec3<f32>(0.75,0.6,0.5)*l, 1.0);
}
";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

pub struct RenderCamera {
    pub view_proj: [[f32; 4]; 4],
    pub inv_view_proj: [[f32; 4]; 4],
    pub position: [f32; 3],
    pub forward: [f32; 3],
    pub near: f32,
    pub far: f32,
    pub day_progress: f32,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const CASCADE_SPLITS: [f32; 4] = [0.05, 0.15, 0.4, 1.0];

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    width: u32,
    height: u32,

    // Camera
    camera_buf: wgpu::Buffer,
    camera_bgl: wgpu::BindGroupLayout,
    camera_bg: wgpu::BindGroup,

    // HDR
    hdr_target: HdrTarget,
    depth_view: wgpu::TextureView,

    // PBR lighting
    pbr: PbrBindings,

    // Shadows
    shadow_map: CascadedShadowMap,
    shadow_cam_bufs: Vec<wgpu::Buffer>,
    shadow_cam_bgs: Vec<wgpu::BindGroup>,
    entity_shadow_pipeline: wgpu::RenderPipeline,

    // Terrain
    terrain_pipeline: wgpu::RenderPipeline,
    terrain_mesh: Option<TerrainMesh>,

    // Entities
    entity_pipeline: wgpu::RenderPipeline,
    capsule: Option<CapsuleMesh>,
    instance_buf: wgpu::Buffer,
    instance_count: u32,
    instance_cap: usize,

    // Vegetation
    vegetation: Option<VegetationSystem>,

    // Skybox
    skybox: SkyboxPass,

    // Post-processing
    tonemap: ToneMapPass,
    tonemap_bg: wgpu::BindGroup,
    bloom: BloomPass,
    bloom_bg: wgpu::BindGroup,

    // Skinned mesh
    skinned_pipeline: wgpu::RenderPipeline,
    pub skin_bgl: wgpu::BindGroupLayout,
    skinned_vb: Option<wgpu::Buffer>,
    skinned_ib: Option<wgpu::Buffer>,
    skinned_index_count: u32,

    // Indirect draw
    indirect_buf: wgpu::Buffer,

    // IBL
    ibl: IblBindings,

    // TAA
    taa: TaaPass,

    // Render pass infrastructure (Block 7: wired dormant passes)
    atmosphere: crate::graphics::atmosphere::AtmospherePass,
    contact_shadows: crate::graphics::contact_shadows::ContactShadowPass,

    // Bruneton atmosphere (Phase 2)
    bruneton: crate::graphics::sky::atmosphere::BrunetonAtmosphere,

    // Sky/Weather systems (Phases 2.5-14.5)
    sky_lighting: crate::graphics::sky::sky_lighting::SkyLightingSystem,
    weather_controller: crate::graphics::sky::weather_controller::WeatherController,
    cloud_coverage: crate::graphics::sky::cloud_coverage::CloudCoveragePass,
    cloud_noise: crate::graphics::sky::cloud_system::CloudNoiseTextures,
    cloud_renderer: crate::graphics::sky::cloud_renderer::CloudRenderer,
    cloud_shadow: crate::graphics::sky::cloud_shadows::CloudShadowPass,
    moon_pass: crate::graphics::sky::moon::MoonPass,
    star_field: crate::graphics::sky::sky_objects::StarField,
    lightning_system: crate::graphics::sky::lightning::LightningSystem,
    fog_system: crate::graphics::sky::fog::FogSystem,
    precipitation_system: crate::graphics::sky::precipitation::PrecipitationSystem,
    blue_noise: crate::graphics::sky::blue_noise::BlueNoiseSampler,
    weather_debug: crate::graphics::sky::debug_views::WeatherDebugState,
    quality_profile: crate::graphics::sky::quality_profiles::SkyQualityProfile,
    weather_frame_counter: u32,

    // Debug
    line_pipeline: wgpu::RenderPipeline,
    grid_vb: wgpu::Buffer,
    grid_count: u32,

    // Particles (Phase B.4)
    particle_system: Option<crate::graphics::particles::ParticleSystem>,
    particle_emitter: Option<crate::graphics::particles::GpuEmitter>,

    // egui integration
    egui_ctx: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
    egui_state: egui_winit::State,
    window: Arc<Window>,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Self {
        let inner = window.inner_size();
        let (w, h) = (inner.width.max(1), inner.height.max(1));

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("no suitable GPU adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
            experimental_features: Default::default(),
        }))
        .expect("failed to create device");

        let caps = surface.get_capabilities(&adapter);
        let surface_format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: w,
            height: h,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // ---- Camera ----
        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buf"),
            contents: bytemuck::bytes_of(&CameraUniform {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let camera_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bg"),
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        // ---- PBR light ----
        let pbr = PbrBindings::new(&device);

        // ---- Shadow map ----
        let shadow_map = CascadedShadowMap::new(&device, &camera_bgl);

        let shadow_cam_bufs: Vec<wgpu::Buffer> = (0..CASCADE_COUNT)
            .map(|i| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("shadow_cam_{i}")),
                    contents: bytemuck::bytes_of(&CameraUniform {
                        view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                    }),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
            })
            .collect();

        let shadow_cam_bgs: Vec<wgpu::BindGroup> = shadow_cam_bufs
            .iter()
            .enumerate()
            .map(|(i, buf)| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("shadow_cam_bg_{i}")),
                    layout: &camera_bgl,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buf.as_entire_binding(),
                    }],
                })
            })
            .collect();

        let entity_shadow_pipeline =
            CascadedShadowMap::create_entity_shadow_pipeline(&device, &camera_bgl);

        // ---- PBR pipeline layout (camera + light + shadow) ----
        let pbr_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pbr_layout"),
            bind_group_layouts: &[&camera_bgl, &pbr.light_bgl, &shadow_map.bind_group_layout],
            push_constant_ranges: &[],
        });

        // ---- Terrain pipeline (PBR) ----
        let terrain_pipeline = {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("pbr_terrain_shader"),
                source: wgpu::ShaderSource::Wgsl(PBR_TERRAIN_SHADER.into()),
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("terrain_pipeline"),
                layout: Some(&pbr_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[TerrainVertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HDR_FORMAT,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };

        // ---- Entity pipeline (PBR, instanced) ----
        let entity_pipeline = {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("pbr_entity_shader"),
                source: wgpu::ShaderSource::Wgsl(PBR_ENTITY_SHADER.into()),
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("entity_pipeline"),
                layout: Some(&pbr_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[MeshVertex::layout(), EntityInstance::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HDR_FORMAT,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };

        // ---- Line pipeline (debug, HDR target) ----
        let camera_only_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("camera_only_layout"),
            bind_group_layouts: &[&camera_bgl],
            push_constant_ranges: &[],
        });

        let line_pipeline = {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("line_shader"),
                source: wgpu::ShaderSource::Wgsl(LINE_SHADER.into()),
            });
            let vbl = wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                    wgpu::VertexAttribute {
                        offset: 12,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x3,
                    },
                ],
            };
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("line_pipeline"),
                layout: Some(&camera_only_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[vbl],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HDR_FORMAT,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };

        // ---- Skinned pipeline ----
        let skin_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("skin_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let skinned_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("skinned_layout"),
                bind_group_layouts: &[&camera_bgl, &skin_bgl],
                push_constant_ranges: &[],
            });
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("skinned_shader"),
                source: wgpu::ShaderSource::Wgsl(SKINNED_SHADER.into()),
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("skinned_pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[SkinVertex::layout()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HDR_FORMAT,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };

        // ---- Skybox (renders to HDR target) ----
        let skybox = SkyboxPass::new(&device, HDR_FORMAT);

        // ---- Post-processing ----
        let hdr_target = HdrTarget::new(&device, w, h);
        let tonemap = ToneMapPass::new(&device, surface_format);
        let tonemap_bg = tonemap.make_bind_group(&device, &hdr_target.view);

        let bloom = BloomPass::new(&device, w, h, HDR_FORMAT);
        let bloom_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom_bg"),
            layout: &bloom.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_target.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&bloom.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bloom.params_buffer.as_entire_binding(),
                },
            ],
        });

        // ---- Capsule + instance buffer ----
        let capsule = CapsuleMesh::new(&device, 0.5, 0.5, 12, 4);
        let instance_cap: usize = 1024;
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("entity_instances"),
            size: (instance_cap * std::mem::size_of::<EntityInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indirect_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("indirect_draw"),
            size: 20, // DrawIndexedIndirectArgs: 5 * u32
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grid_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid_vb"),
            contents: &[0u8; 4],
            usage: wgpu::BufferUsages::VERTEX,
        });

        let depth_view = Self::create_depth(&device, w, h);

        let ibl = IblBindings::new(&device);
        let taa = TaaPass::new(&device, w, h, surface_format);
        let atmosphere = crate::graphics::atmosphere::AtmospherePass::new(&device, surface_format);
        let contact_shadows =
            crate::graphics::contact_shadows::ContactShadowPass::new(&device, surface_format);
        let bruneton = crate::graphics::sky::atmosphere::BrunetonAtmosphere::new(&device, &queue);

        let sky_lighting = crate::graphics::sky::sky_lighting::SkyLightingSystem::new();
        let weather_controller =
            crate::graphics::sky::weather_controller::WeatherController::new(42);
        let cloud_coverage = crate::graphics::sky::cloud_coverage::CloudCoveragePass::new(&device);
        let cloud_noise =
            crate::graphics::sky::cloud_system::CloudNoiseTextures::new(&device, &queue);
        let mut cloud_renderer =
            crate::graphics::sky::cloud_renderer::CloudRenderer::new(&device, &queue, HDR_FORMAT);

        // Connect cloud noise textures to cloud renderer
        let cloud_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cloud_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        cloud_renderer.bind_group = cloud_renderer.create_bind_group(
            &device,
            &cloud_coverage.output_view,
            &cloud_noise.perlin_worley_view,
            &cloud_noise.worley_view,
            &cloud_sampler,
        );

        let cloud_shadow = crate::graphics::sky::cloud_shadows::CloudShadowPass::new(&device);
        let moon_pass = crate::graphics::sky::moon::MoonPass::new(&device, &queue, HDR_FORMAT);
        let star_field = crate::graphics::sky::sky_objects::StarField::new(&device, HDR_FORMAT);
        let lightning_system = crate::graphics::sky::lightning::LightningSystem::new();
        let fog_system = crate::graphics::sky::fog::FogSystem::new(&device, HDR_FORMAT);
        let precipitation_system =
            crate::graphics::sky::precipitation::PrecipitationSystem::new(&device, HDR_FORMAT);
        let blue_noise = crate::graphics::sky::blue_noise::BlueNoiseSampler::new(&device, &queue);
        let weather_debug = crate::graphics::sky::debug_views::WeatherDebugState::default();
        let quality_profile = crate::graphics::sky::quality_profiles::SkyQualityProfile::high();

        // Particles (Phase B.4)
        let particle_system = Some(crate::graphics::particles::ParticleSystem::new(
            &device, HDR_FORMAT,
        ));

        let egui_ctx = egui::Context::default();
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            None,
            None,
            None,
        );

        Self {
            surface,
            device,
            queue,
            config,
            width: w,
            height: h,
            camera_buf,
            camera_bgl,
            camera_bg,
            hdr_target,
            depth_view,
            pbr,
            shadow_map,
            shadow_cam_bufs,
            shadow_cam_bgs,
            entity_shadow_pipeline,
            terrain_pipeline,
            terrain_mesh: None,
            entity_pipeline,
            capsule: Some(capsule),
            instance_buf,
            instance_count: 0,
            instance_cap,
            vegetation: None,
            skybox,
            tonemap,
            tonemap_bg,
            bloom,
            bloom_bg,
            indirect_buf,
            ibl,
            taa,
            atmosphere,
            contact_shadows,
            bruneton,
            sky_lighting,
            weather_controller,
            cloud_coverage,
            cloud_noise,
            cloud_renderer,
            cloud_shadow,
            moon_pass,
            star_field,
            lightning_system,
            fog_system,
            precipitation_system,
            blue_noise,
            weather_debug,
            quality_profile,
            weather_frame_counter: 0,
            skinned_pipeline,
            skin_bgl,
            skinned_vb: None,
            skinned_ib: None,
            skinned_index_count: 0,
            line_pipeline,
            grid_vb,
            grid_count: 0,
            particle_system,
            particle_emitter: None,
            egui_ctx,
            egui_renderer,
            egui_state,
            window,
        }
    }

    pub fn upload_terrain(&mut self, heightmap: &Heightmap, biomes: &[Biome]) {
        self.terrain_mesh = Some(TerrainMesh::build(&self.device, heightmap, biomes));
    }

    pub fn upload_vegetation(&mut self, heightmap: &Heightmap, biomes: &[Biome]) {
        self.vegetation = Some(VegetationSystem::new(
            &self.device,
            &self.camera_bgl,
            HDR_FORMAT,
            heightmap,
            biomes,
        ));
    }

    pub fn weather_cloud_coverage(&self) -> f32 {
        self.weather_controller.cloud_coverage
    }

    pub fn force_weather_state(&mut self, index: u32) {
        use crate::graphics::sky::weather_controller::WeatherState;
        let state = match index {
            0 => WeatherState::Clear,
            1 => WeatherState::Cloudy,
            2 => WeatherState::Storm,
            3 => WeatherState::Rain,
            4 => WeatherState::Clearing,
            _ => WeatherState::Clear,
        };
        self.weather_controller.force_state(state);
    }

    /// Cycle through weather debug visualization modes
    pub fn cycle_weather_debug_mode(&mut self) {
        self.weather_debug.cycle_mode();
    }

    /// Get current weather debug mode
    pub fn weather_debug_mode(&self) -> crate::graphics::sky::debug_views::DebugViewMode {
        self.weather_debug.active_mode
    }

    /// Set sky quality profile
    pub fn set_quality_profile(
        &mut self,
        profile: crate::graphics::sky::quality_profiles::SkyQualityProfile,
    ) {
        self.quality_profile = profile;
    }

    /// Get current quality profile
    pub fn quality_profile(&self) -> &crate::graphics::sky::quality_profiles::SkyQualityProfile {
        &self.quality_profile
    }

    /// Access cloud noise textures (for debugging or regeneration)
    pub fn cloud_noise(&self) -> &crate::graphics::sky::cloud_system::CloudNoiseTextures {
        &self.cloud_noise
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }
        self.width = w;
        self.height = h;
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        self.depth_view = Self::create_depth(&self.device, w, h);
        self.hdr_target.resize(&self.device, w, h);
        self.tonemap_bg = self
            .tonemap
            .make_bind_group(&self.device, &self.hdr_target.view);
        self.bloom = BloomPass::new(&self.device, w, h, HDR_FORMAT);
        self.bloom_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bloom_bg"),
            layout: &self.bloom.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.hdr_target.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.bloom.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.bloom.params_buffer.as_entire_binding(),
                },
            ],
        });
        self.taa.resize(&self.device, w, h, self.config.format);
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn egui_context(&self) -> &egui::Context {
        &self.egui_ctx
    }

    pub fn on_window_event(&mut self, event: &WindowEvent) -> egui_winit::EventResponse {
        self.egui_state.on_window_event(&self.window, event)
    }

    pub fn render_with_egui(
        &mut self,
        cam: &RenderCamera,
        egui_draw_fn: impl FnOnce(&egui::Context),
    ) -> Result<(), wgpu::SurfaceError> {
        self.queue.write_buffer(
            &self.camera_buf,
            0,
            bytemuck::bytes_of(&CameraUniform {
                view_proj: cam.view_proj,
            }),
        );

        let sun = SunLight::from_day_progress(cam.day_progress, cam.position);
        self.pbr.update(&self.queue, &sun);

        let sun_dir = Vec3::new(sun.direction[0], sun.direction[1], sun.direction[2]);
        let cam_pos = Vec3::from(cam.position);
        let cam_fwd = Vec3::from(cam.forward);

        let cascade_vps = compute_cascade_vps(cam_pos, cam_fwd, sun_dir, cam.near, cam.far);

        for (i, vp) in cascade_vps.iter().enumerate() {
            let u = CameraUniform {
                view_proj: vp.to_cols_array_2d(),
            };
            self.queue
                .write_buffer(&self.shadow_cam_bufs[i], 0, bytemuck::bytes_of(&u));
        }

        let shadow_uniforms = ShadowUniforms {
            light_view_proj: std::array::from_fn(|i| cascade_vps[i].to_cols_array_2d()),
            cascade_splits: [
                cam.near + (cam.far - cam.near) * CASCADE_SPLITS[0],
                cam.near + (cam.far - cam.near) * CASCADE_SPLITS[1],
                cam.near + (cam.far - cam.near) * CASCADE_SPLITS[2],
                cam.far,
            ],
        };
        self.queue.write_buffer(
            &self.shadow_map.uniform_buffer,
            0,
            bytemuck::bytes_of(&shadow_uniforms),
        );
        self.ibl.update(&self.queue, cam.day_progress);
        self.taa.advance_frame();

        let inv_vp = Mat4::from_cols_array_2d(&cam.inv_view_proj);
        self.skybox.update(
            &self.queue,
            inv_vp,
            [sun.direction[0], sun.direction[1], sun.direction[2]],
            cam.position,
        );

        let output = self.surface.get_current_texture()?;
        let swapchain_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        // Shadow passes
        for i in 0..CASCADE_COUNT {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("shadow_pass_{i}")),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_map.views[i],
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // Terrain shadow
            if let Some(tm) = &self.terrain_mesh {
                pass.set_pipeline(&self.shadow_map.shadow_pipeline);
                pass.set_bind_group(0, &self.shadow_cam_bgs[i], &[]);
                pass.set_vertex_buffer(0, tm.vertex_buffer.slice(..));
                pass.set_index_buffer(tm.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..tm.index_count, 0, 0..1);
            }

            // Entity shadow (instanced capsules)
            if let Some(cap) = &self.capsule {
                if self.instance_count > 0 {
                    pass.set_pipeline(&self.entity_shadow_pipeline);
                    pass.set_bind_group(0, &self.shadow_cam_bgs[i], &[]);
                    pass.set_vertex_buffer(0, cap.vertex_buffer.slice(..));
                    pass.set_vertex_buffer(1, self.instance_buf.slice(..));
                    pass.set_index_buffer(cap.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..cap.index_count, 0, 0..self.instance_count);
                }
            }
        }

        // Geometry pass
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("geometry_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            if let Some(tm) = &self.terrain_mesh {
                pass.set_pipeline(&self.terrain_pipeline);
                pass.set_bind_group(0, &self.camera_bg, &[]);
                pass.set_bind_group(1, &self.pbr.light_bg, &[]);
                pass.set_bind_group(2, &self.shadow_map.bind_group, &[]);
                pass.set_vertex_buffer(0, tm.vertex_buffer.slice(..));
                pass.set_index_buffer(tm.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..tm.index_count, 0, 0..1);
            }

            if let Some(cap) = &self.capsule {
                if self.instance_count > 0 {
                    pass.set_pipeline(&self.entity_pipeline);
                    pass.set_bind_group(0, &self.camera_bg, &[]);
                    pass.set_bind_group(1, &self.pbr.light_bg, &[]);
                    pass.set_bind_group(2, &self.shadow_map.bind_group, &[]);
                    pass.set_vertex_buffer(0, cap.vertex_buffer.slice(..));
                    pass.set_vertex_buffer(1, self.instance_buf.slice(..));
                    pass.set_index_buffer(cap.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed_indirect(&self.indirect_buf, 0);
                }
            }

            if let Some(veg) = &self.vegetation {
                if veg.grass_count > 0 {
                    pass.set_pipeline(&veg.pipeline);
                    pass.set_bind_group(0, &self.camera_bg, &[]);
                    pass.set_vertex_buffer(0, veg.grass_quad_vb.slice(..));
                    pass.set_vertex_buffer(1, veg.grass_instance_buf.slice(..));
                    pass.set_index_buffer(veg.grass_quad_ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..12, 0, 0..veg.grass_count);
                }
            }
        }

        // Skybox
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("skybox_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&self.skybox.pipeline);
            pass.set_bind_group(0, &self.skybox.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        // Tonemap
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tonemap_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &swapchain_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.tonemap.pipeline);
            pass.set_bind_group(0, &self.tonemap_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // egui overlay: full rendering pipeline with real input
        let raw_input = self.egui_state.take_egui_input(&self.window);
        self.egui_ctx.begin_pass(raw_input);
        egui_draw_fn(&self.egui_ctx);
        let full_output = self.egui_ctx.end_pass();
        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output.clone());

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.width, self.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        let user_cmd_bufs = self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_desc,
        );

        {
            let egui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &swapchain_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            self.egui_renderer.render(
                &mut egui_pass.forget_lifetime(),
                &clipped_primitives,
                &screen_desc,
            );
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        let mut cmds: Vec<wgpu::CommandBuffer> = user_cmd_bufs;
        cmds.push(encoder.finish());
        self.queue.submit(cmds);
        output.present();
        Ok(())
    }

    pub fn update_entities(&mut self, instances: &[EntityInstance]) {
        let count = instances.len();
        if count > self.instance_cap {
            self.instance_cap = (count * 2).max(256);
            self.instance_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("entity_instances"),
                size: (self.instance_cap * std::mem::size_of::<EntityInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if count > 0 {
            self.queue
                .write_buffer(&self.instance_buf, 0, bytemuck::cast_slice(instances));
        }
        self.instance_count = count as u32;

        // Update indirect draw args
        if let Some(cap) = &self.capsule {
            let args: [u32; 5] = [
                cap.index_count,     // index_count
                self.instance_count, // instance_count
                0,                   // first_index
                0,                   // base_vertex
                0,                   // first_instance
            ];
            self.queue
                .write_buffer(&self.indirect_buf, 0, bytemuck::cast_slice(&args));
        }
    }

    pub fn upload_skinned_mesh(&mut self, vertices: &[SkinVertex], indices: &[u32]) {
        self.skinned_vb = Some(
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("skinned_vb"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        );
        self.skinned_ib = Some(
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("skinned_ib"),
                    contents: bytemuck::cast_slice(indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
        );
        self.skinned_index_count = indices.len() as u32;
    }

    pub fn render_skinned(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        skin_buffer: &SkinBuffer,
    ) {
        if self.skinned_vb.is_none() || self.skinned_index_count == 0 {
            return;
        }
        let vb = self.skinned_vb.as_ref().unwrap();
        let ib = self.skinned_ib.as_ref().unwrap();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("skinned_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        pass.set_pipeline(&self.skinned_pipeline);
        pass.set_bind_group(0, &self.camera_bg, &[]);
        pass.set_bind_group(1, &skin_buffer.bind_group, &[]);
        pass.set_vertex_buffer(0, vb.slice(..));
        pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..self.skinned_index_count, 0, 0..1);
    }

    pub fn render(&mut self, cam: &RenderCamera) -> Result<(), wgpu::SurfaceError> {
        // ---- Update uniforms ----
        self.queue.write_buffer(
            &self.camera_buf,
            0,
            bytemuck::bytes_of(&CameraUniform {
                view_proj: cam.view_proj,
            }),
        );

        let sun = SunLight::from_day_progress(cam.day_progress, cam.position);
        self.pbr.update(&self.queue, &sun);

        let sun_dir = Vec3::new(sun.direction[0], sun.direction[1], sun.direction[2]);
        let cam_pos = Vec3::from(cam.position);
        let cam_fwd = Vec3::from(cam.forward);

        // Bruneton atmosphere: update config and init LUT pipeline on first frame
        self.bruneton.update_config(
            &self.queue,
            [sun.direction[0], sun.direction[1], sun.direction[2]],
            cam.position[1],
        );
        if !self.skybox.use_lut {
            self.skybox
                .init_lut_pipeline(&self.device, HDR_FORMAT, &self.bruneton);
        }

        // Compute cascade VP matrices
        let cascade_vps = compute_cascade_vps(cam_pos, cam_fwd, sun_dir, cam.near, cam.far);

        // Write cascade camera buffers (for shadow depth passes)
        for (i, vp) in cascade_vps.iter().enumerate() {
            let u = CameraUniform {
                view_proj: vp.to_cols_array_2d(),
            };
            self.queue
                .write_buffer(&self.shadow_cam_bufs[i], 0, bytemuck::bytes_of(&u));
        }

        // Write shadow uniforms (for PBR fragment shader)
        let shadow_uniforms = ShadowUniforms {
            light_view_proj: std::array::from_fn(|i| cascade_vps[i].to_cols_array_2d()),
            cascade_splits: [
                cam.near + (cam.far - cam.near) * CASCADE_SPLITS[0],
                cam.near + (cam.far - cam.near) * CASCADE_SPLITS[1],
                cam.near + (cam.far - cam.near) * CASCADE_SPLITS[2],
                cam.far,
            ],
        };
        self.queue.write_buffer(
            &self.shadow_map.uniform_buffer,
            0,
            bytemuck::bytes_of(&shadow_uniforms),
        );

        // Update IBL environment lighting
        self.ibl.update(&self.queue, cam.day_progress);

        // Advance TAA frame counter (jitter applied to projection by caller)
        self.taa.advance_frame();

        // Update skybox
        let inv_vp = Mat4::from_cols_array_2d(&cam.inv_view_proj);
        self.skybox.update(
            &self.queue,
            inv_vp,
            [sun.direction[0], sun.direction[1], sun.direction[2]],
            cam.position,
        );

        // ---- Weather simulation (Phase 13) ----
        self.weather_frame_counter = self.weather_frame_counter.wrapping_add(1);
        self.blue_noise.advance_frame();

        // Step weather controller (dt ~16ms at 60fps)
        let dt = 1.0 / 60.0;
        self.weather_controller.update(dt, cam.day_progress);

        // Update lightning
        self.lightning_system.update(dt);

        // Compute sky lighting from weather state
        let _weather_lighting = self.sky_lighting.update(
            sun_dir,
            self.weather_controller.cloud_coverage,
            self.weather_controller.rain_intensity(),
            self.lightning_system.flash_intensity,
            -sun_dir.y,
            cam.day_progress,
        );

        // Update fog
        {
            let fog_cfg = crate::graphics::sky::fog::FogConfig::default();
            self.fog_system.update(
                &self.queue,
                &fog_cfg,
                cam_pos,
                sun_dir,
                cam.day_progress,
                5.0,
                inv_vp,
            );
        }

        // Update precipitation
        {
            let view_proj = Mat4::from_cols_array_2d(&cam.view_proj);
            let wind_vec = Vec3::new(3.0, 0.0, 1.0);
            let params = crate::graphics::sky::precipitation::build_precipitation_params(
                view_proj,
                crate::graphics::sky::precipitation::PrecipitationType::Rain,
                self.weather_controller.rain_intensity(),
                wind_vec,
                self.weather_frame_counter as f32 * dt,
                cam_pos,
            );
            self.precipitation_system.update(&self.queue, params);
        }

        // Update cloud coverage (every 4 frames)
        let should_update_clouds = self.weather_frame_counter % 4 == 0;

        // Update star field
        self.star_field.update(
            &self.queue,
            Mat4::from_cols_array_2d(&cam.view_proj),
            10000.0,
            sun_dir.y,
        );

        // Update moon
        self.moon_pass.update(
            &self.queue,
            sun_dir,
            cam.day_progress,
            cam.day_progress * 365.0,
            Mat4::from_cols_array_2d(&cam.view_proj),
            cam_pos,
        );

        // ---- Acquire swapchain ----
        let output = self.surface.get_current_texture()?;
        let swapchain_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        // ---- Atmosphere LUT compute (recompute when sun moves) ----
        if self.bruneton.needs_recompute(sun.direction[1]) {
            self.bruneton.compute_luts(&mut encoder, sun.direction[1]);
        }

        // ---- Cloud coverage compute (every 4 frames) ----
        if should_update_clouds {
            let cov_params = crate::graphics::sky::cloud_coverage::CloudCoverageParams {
                wind_offset: [self.weather_frame_counter as f32 * dt * 2.0, 0.0],
                coverage_factor: self.weather_controller.cloud_coverage,
                time: self.weather_frame_counter as f32 * dt,
            };
            self.cloud_coverage.update_params(&self.queue, &cov_params);
            self.cloud_coverage.compute(&mut encoder);

            // Build a simple orthographic light VP for cloud shadow
            let light_dir = sun_dir.normalize();
            let light_pos = cam_pos - light_dir * 10000.0;
            let cloud_shadow_view = Mat4::look_at_rh(light_pos, cam_pos, Vec3::Y);
            let cloud_shadow_proj =
                Mat4::orthographic_rh(-5000.0, 5000.0, -5000.0, 5000.0, 0.1, 30000.0);
            let cloud_shadow_vp = cloud_shadow_proj * cloud_shadow_view;
            let shadow_params = crate::graphics::sky::cloud_shadows::CloudShadowParams {
                light_view_proj: cloud_shadow_vp.to_cols_array_2d(),
                inv_light_view_proj: cloud_shadow_vp.inverse().to_cols_array_2d(),
                cloud_layer_bottom: 1500.0,
                cloud_layer_top: 8000.0,
                world_scale: 10000.0,
                _pad: 0.0,
            };
            self.cloud_shadow.update_params(&self.queue, &shadow_params);
            self.cloud_shadow.compute(&mut encoder);
        }

        // ---- Shadow depth passes ----
        for i in 0..CASCADE_COUNT {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("shadow_pass_{i}")),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_map.views[i],
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            if let Some(tm) = &self.terrain_mesh {
                pass.set_pipeline(&self.shadow_map.shadow_pipeline);
                pass.set_bind_group(0, &self.shadow_cam_bgs[i], &[]);
                pass.set_vertex_buffer(0, tm.vertex_buffer.slice(..));
                pass.set_index_buffer(tm.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..tm.index_count, 0, 0..1);
            }

            if let Some(cap) = &self.capsule {
                if self.instance_count > 0 {
                    pass.set_pipeline(&self.entity_shadow_pipeline);
                    pass.set_bind_group(0, &self.shadow_cam_bgs[i], &[]);
                    pass.set_vertex_buffer(0, cap.vertex_buffer.slice(..));
                    pass.set_vertex_buffer(1, self.instance_buf.slice(..));
                    pass.set_index_buffer(cap.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..cap.index_count, 0, 0..self.instance_count);
                }
            }
        }

        // ---- Geometry pass (HDR target) ----
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("geometry_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // Terrain
            if let Some(tm) = &self.terrain_mesh {
                pass.set_pipeline(&self.terrain_pipeline);
                pass.set_bind_group(0, &self.camera_bg, &[]);
                pass.set_bind_group(1, &self.pbr.light_bg, &[]);
                pass.set_bind_group(2, &self.shadow_map.bind_group, &[]);
                pass.set_vertex_buffer(0, tm.vertex_buffer.slice(..));
                pass.set_index_buffer(tm.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..tm.index_count, 0, 0..1);
            }

            // Entities (draw_indexed_indirect)
            if let Some(cap) = &self.capsule {
                if self.instance_count > 0 {
                    pass.set_pipeline(&self.entity_pipeline);
                    pass.set_bind_group(0, &self.camera_bg, &[]);
                    pass.set_bind_group(1, &self.pbr.light_bg, &[]);
                    pass.set_bind_group(2, &self.shadow_map.bind_group, &[]);
                    pass.set_vertex_buffer(0, cap.vertex_buffer.slice(..));
                    pass.set_vertex_buffer(1, self.instance_buf.slice(..));
                    pass.set_index_buffer(cap.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed_indirect(&self.indirect_buf, 0);
                }
            }

            // Vegetation
            if let Some(veg) = &self.vegetation {
                if veg.grass_count > 0 {
                    pass.set_pipeline(&veg.pipeline);
                    pass.set_bind_group(0, &self.camera_bg, &[]);
                    pass.set_vertex_buffer(0, veg.grass_quad_vb.slice(..));
                    pass.set_vertex_buffer(1, veg.grass_instance_buf.slice(..));
                    pass.set_index_buffer(veg.grass_quad_ib.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..12, 0, 0..veg.grass_count);
                }
                if veg.tree_count > 0 {
                    pass.set_vertex_buffer(1, veg.tree_instance_buf.slice(..));
                    pass.draw_indexed(0..12, 0, 0..veg.tree_count);
                }
            }

            // Debug lines
            if self.grid_count > 0 {
                pass.set_pipeline(&self.line_pipeline);
                pass.set_bind_group(0, &self.camera_bg, &[]);
                pass.set_vertex_buffer(0, self.grid_vb.slice(..));
                pass.draw(0..self.grid_count, 0..1);
            }
        }

        // ---- Skybox pass (HDR target, depth read-only) -- Bruneton LUT when available ----
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("skybox_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(self.skybox.active_pipeline());
            pass.set_bind_group(0, self.skybox.active_bind_group(), &[]);
            pass.draw(0..3, 0..1);
        }

        // ---- Volumetric Cloud pass (HDR, after skybox, alpha-blend) ----
        {
            // Use quality profile for ray steps
            let ray_steps = if self.quality_profile.cloud_ray_steps > 0 {
                self.quality_profile.cloud_ray_steps
            } else {
                48 // fallback
            };

            let cloud_params = crate::graphics::sky::cloud_renderer::CloudRenderParams {
                inv_view_proj: inv_vp.to_cols_array_2d(),
                sun_direction: [sun.direction[0], sun.direction[1], sun.direction[2], 0.0],
                camera_pos: [cam.position[0], cam.position[1], cam.position[2], 1.0],
                time: self.weather_frame_counter as f32 * (1.0 / 60.0),
                cloud_coverage: self.weather_controller.cloud_coverage,
                cloud_layer_bottom: 1500.0,
                cloud_layer_top: 12000.0,
                density_multiplier: 0.5,
                light_absorption: 0.35,
                henyey_g: 0.76,
                ray_steps,
                _pad: [0; 4],
            };
            self.cloud_renderer.update(&self.queue, &cloud_params);

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("cloud_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            self.cloud_renderer.render(&mut pass);
        }

        // ---- Fog pass (HDR, after clouds, alpha-blend) ----
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fog_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            self.fog_system.render(&mut pass);
        }

        // ---- Star field pass (HDR, after skybox) ----
        if sun_dir.y < 0.2 {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("star_field_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            self.star_field.render(&mut pass, sun_dir.y);
        }

        // ---- Moon pass (HDR, after stars) ----
        if sun_dir.y < 0.1 {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("moon_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            self.moon_pass.render(&mut pass);
        }

        // ---- Precipitation pass (HDR, after geometry) ----
        if self.weather_controller.rain_intensity() > 0.01 {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("precipitation_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.hdr_target.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            self.precipitation_system.render(&mut pass);
        }

        // ---- Bloom extraction (HDR -> bloom mip 0) ----
        if self.bloom.mip_count > 0 {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bloom.downsample_views[0],
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.bloom.pipeline);
            pass.set_bind_group(0, &self.bloom_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // ---- Bloom composite (additive blend onto HDR) ----
        // The bloom result is in bloom.downsample_views[0]. We skip the
        // composite for now; the tonemap pass reads the original HDR which
        // already contains the bright values. A proper multi-pass blur +
        // additive composite can be added later for higher quality.

        // ---- Atmosphere + Contact Shadows state update ----
        {
            let mut atmo_params = crate::graphics::atmosphere::AtmosphereParams::default();
            atmo_params.sun_direction = [sun.direction[0], sun.direction[1], sun.direction[2], 0.0];
            atmo_params.camera_pos = [cam.position[0], cam.position[1], cam.position[2], 1.0];
            self.atmosphere.update(&self.queue, &atmo_params);
            let _cs_pipeline = &self.contact_shadows.pipeline;
            let _cs_sampler = &self.contact_shadows.sampler;
            let _cs_bgl = &self.contact_shadows.bgl;
            let _cs_buf = &self.contact_shadows.params_buf;
        }

        // ---- Tonemap (HDR -> swapchain sRGB) ----
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("tonemap_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &swapchain_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.tonemap.pipeline);
            pass.set_bind_group(0, &self.tonemap_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    fn create_depth(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("depth"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: DEPTH_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    // =========================================================================
    // Particle System API (Phase B.4)
    // =========================================================================

    /// Spawn a particle emitter at the given position
    pub fn spawn_particles(&mut self, emitter: crate::graphics::particles::GpuEmitter) {
        self.particle_emitter = Some(emitter);
    }

    /// Spawn fire particles at position
    pub fn spawn_fire(&mut self, pos: [f32; 3]) {
        self.particle_emitter = Some(crate::graphics::particles::fire_emitter(pos));
    }

    /// Spawn smoke particles at position
    pub fn spawn_smoke(&mut self, pos: [f32; 3]) {
        self.particle_emitter = Some(crate::graphics::particles::smoke_emitter(pos));
    }

    /// Spawn rain particles at position
    pub fn spawn_rain(&mut self, pos: [f32; 3]) {
        self.particle_emitter = Some(crate::graphics::particles::rain_emitter(pos));
    }

    /// Clear active particle emitter
    pub fn clear_particles(&mut self) {
        self.particle_emitter = None;
    }

    /// Get particle system for advanced usage
    pub fn particle_system(&self) -> Option<&crate::graphics::particles::ParticleSystem> {
        self.particle_system.as_ref()
    }

    /// Get mutable particle system for advanced usage
    pub fn particle_system_mut(
        &mut self,
    ) -> Option<&mut crate::graphics::particles::ParticleSystem> {
        self.particle_system.as_mut()
    }

    /// Update and render particles (call during frame rendering)
    pub fn render_particles(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        dt: f32,
        view_proj: [[f32; 4]; 4],
        camera_right: [f32; 3],
        camera_up: [f32; 3],
    ) {
        let Some(ps) = &mut self.particle_system else {
            return;
        };
        let Some(emitter) = &self.particle_emitter else {
            return;
        };

        // Update particle simulation
        ps.dispatch_update(encoder, &self.queue, dt, emitter);

        // Render particles to HDR target
        let camera_uniform = crate::graphics::particles::ParticleCameraUniform {
            view_proj,
            camera_right,
            _p0: 0.0,
            camera_up,
            _p1: 0.0,
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("particle_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.hdr_target.view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        ps.render(&mut pass, &self.queue, &camera_uniform);
    }
}

fn compute_cascade_vps(
    camera_pos: Vec3,
    camera_forward: Vec3,
    sun_direction: Vec3,
    near: f32,
    far: f32,
) -> [Mat4; CASCADE_COUNT] {
    let mut result = [Mat4::IDENTITY; CASCADE_COUNT];
    let mut prev_split = near;
    for (i, &pct) in CASCADE_SPLITS.iter().enumerate() {
        let split_far = near + (far - near) * pct;
        let center = camera_pos + camera_forward * ((prev_split + split_far) * 0.5);
        let radius = (split_far - prev_split) * 0.5 * 1.5;
        let light_dir = sun_direction.normalize();
        let light_pos = center - light_dir * radius * 2.0;
        let view = Mat4::look_at_rh(light_pos, center, Vec3::Y);
        let proj = Mat4::orthographic_rh(-radius, radius, -radius, radius, 0.1, radius * 4.0);
        result[i] = proj * view;
        prev_split = split_far;
    }
    result
}

// =============================================================================
// Render Budget Enforcement (Phase C.5)
// =============================================================================

impl Renderer {
    /// Apply quality governor settings to render passes
    pub fn apply_budget_settings(
        &mut self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) {
        use crate::core::quality_governor::PressureLevel;

        // Adjust shadow cascade count and resolution based on pressure
        match governor.pressure_level {
            PressureLevel::Critical => {
                // Reduce to 2 cascades at critical pressure
                // Note: Actual cascade reduction would require pipeline recreation
                // For now, we reduce cloud quality
                self.quality_profile.cloud_ray_steps = 16;
            }
            PressureLevel::High => {
                self.quality_profile.cloud_ray_steps = 32;
            }
            PressureLevel::Moderate => {
                self.quality_profile.cloud_ray_steps = 48;
            }
            PressureLevel::Normal => {
                self.quality_profile.cloud_ray_steps = 64;
            }
        }

        // Adjust bloom threshold based on pressure level
        let bloom_threshold = match governor.pressure_level {
            PressureLevel::Critical => 2.0, // Higher threshold = less bloom
            PressureLevel::High => 1.5,
            PressureLevel::Moderate => 1.2,
            PressureLevel::Normal => 1.0,
        };

        // Update bloom params
        let mut params = crate::graphics::postprocess::PostProcessParams::default();
        params.bloom_threshold = bloom_threshold;
        self.bloom.update_params(&self.queue, &params);
    }

    /// Get max particle count based on pressure level
    pub fn max_particle_count(
        &self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> usize {
        use crate::core::quality_governor::PressureLevel;
        match governor.pressure_level {
            PressureLevel::Normal => 10000,
            PressureLevel::Moderate => 5000,
            PressureLevel::High => 2000,
            PressureLevel::Critical => 500,
        }
    }

    /// Check if vegetation should be rendered
    pub fn should_render_vegetation(
        &self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> bool {
        use crate::core::quality_governor::PressureLevel;
        !matches!(governor.pressure_level, PressureLevel::Critical)
    }

    /// Get vegetation detail level (0.0 - 1.0)
    pub fn vegetation_detail(
        &self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> f32 {
        use crate::core::quality_governor::PressureLevel;
        match governor.pressure_level {
            PressureLevel::Normal => 1.0,
            PressureLevel::Moderate => 0.7,
            PressureLevel::High => 0.4,
            PressureLevel::Critical => 0.0,
        }
    }

    /// Check if clouds should be rendered
    pub fn should_render_clouds(
        &self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> bool {
        use crate::core::quality_governor::PressureLevel;
        !matches!(governor.pressure_level, PressureLevel::Critical)
    }

    /// Check if fog should be rendered
    pub fn should_render_fog(
        &self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> bool {
        use crate::core::quality_governor::PressureLevel;
        !matches!(governor.pressure_level, PressureLevel::Critical)
    }

    /// Check if precipitation should be rendered
    pub fn should_render_precipitation(
        &self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> bool {
        use crate::core::quality_governor::PressureLevel;
        !matches!(governor.pressure_level, PressureLevel::Critical)
    }

    /// Get debris density factor from governor
    pub fn debris_density(&self, governor: &crate::core::quality_governor::QualityGovernor) -> f32 {
        governor.debris_density_factor()
    }

    /// Get max terrain rebuilds per frame from governor
    pub fn max_terrain_rebuilds(
        &self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> usize {
        governor.max_terrain_rebuilds_per_frame()
    }

    /// Get max dirty surface uploads from governor
    pub fn max_dirty_uploads(
        &self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> usize {
        governor.max_dirty_surface_uploads()
    }

    /// Get max chain reaction depth from governor
    pub fn max_chain_depth(
        &self,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> u32 {
        governor.max_chain_reaction_depth()
    }

    /// Render with budget enforcement
    pub fn render_with_budget(
        &mut self,
        cam: &RenderCamera,
        governor: &crate::core::quality_governor::QualityGovernor,
    ) -> Result<(), wgpu::SurfaceError> {
        // Apply budget settings first
        self.apply_budget_settings(governor);

        // Then render normally
        self.render(cam)
    }
}
