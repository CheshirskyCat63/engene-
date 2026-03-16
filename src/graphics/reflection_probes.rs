use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ProbeData {
    pub position: [f32; 3],
    pub radius: f32,
    pub irradiance_sh: [[f32; 4]; 9],
}

pub struct ReflectionProbe {
    pub position: [f32; 3],
    pub radius: f32,
    pub cubemap: Option<wgpu::Texture>,
    pub cubemap_view: Option<wgpu::TextureView>,
}

pub struct ReflectionProbeSystem {
    probes: Vec<ReflectionProbe>,
    probe_buffer: wgpu::Buffer,
    max_probes: usize,
}

impl ReflectionProbeSystem {
    pub fn new(device: &wgpu::Device, max_probes: usize) -> Self {
        let buf_size = (max_probes * std::mem::size_of::<ProbeData>()) as u64;
        let probe_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("probe_data"),
            size: buf_size.max(64),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            probes: Vec::new(),
            probe_buffer,
            max_probes,
        }
    }

    pub fn add_probe(&mut self, position: [f32; 3], radius: f32) {
        if self.probes.len() >= self.max_probes {
            return;
        }
        self.probes.push(ReflectionProbe {
            position,
            radius,
            cubemap: None,
            cubemap_view: None,
        });
    }

    pub fn nearest_probe(&self, pos: [f32; 3]) -> Option<usize> {
        let mut best = None;
        let mut best_dist = f32::MAX;
        for (i, probe) in self.probes.iter().enumerate() {
            let dx = pos[0] - probe.position[0];
            let dy = pos[1] - probe.position[1];
            let dz = pos[2] - probe.position[2];
            let dist = dx * dx + dy * dy + dz * dz;
            if dist < best_dist && dist < probe.radius * probe.radius {
                best_dist = dist;
                best = Some(i);
            }
        }
        best
    }

    pub fn probe_count(&self) -> usize {
        self.probes.len()
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.probe_buffer
    }
}
