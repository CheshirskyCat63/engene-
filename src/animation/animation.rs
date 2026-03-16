use glam::{Mat4, Quat, Vec3};

pub struct Skeleton {
    pub joints: Vec<Joint>,
    pub inverse_bind_matrices: Vec<Mat4>,
}

pub struct Joint {
    pub name: String,
    pub parent: Option<usize>,
    pub local_bind_transform: Mat4,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            joints: Vec::new(),
            inverse_bind_matrices: Vec::new(),
        }
    }

    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    pub fn compute_world_transforms(&self, local_transforms: &[Mat4]) -> Vec<Mat4> {
        let n = self.joints.len();
        let mut world = vec![Mat4::IDENTITY; n];
        for i in 0..n {
            let local = if i < local_transforms.len() {
                local_transforms[i]
            } else {
                self.joints[i].local_bind_transform
            };
            world[i] = match self.joints[i].parent {
                Some(p) => world[p] * local,
                None => local,
            };
        }
        world
    }

    pub fn compute_skin_matrices(&self, local_transforms: &[Mat4]) -> Vec<Mat4> {
        let world = self.compute_world_transforms(local_transforms);
        world
            .iter()
            .enumerate()
            .map(|(i, w)| {
                if i < self.inverse_bind_matrices.len() {
                    *w * self.inverse_bind_matrices[i]
                } else {
                    *w
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChannelProperty {
    Translation,
    Rotation,
    Scale,
}

pub struct Keyframe {
    pub time: f32,
    pub value: [f32; 4],
}

pub struct Channel {
    pub joint_index: usize,
    pub property: ChannelProperty,
    pub keyframes: Vec<Keyframe>,
}

impl Channel {
    pub fn sample(&self, t: f32) -> [f32; 4] {
        if self.keyframes.is_empty() {
            return [0.0; 4];
        }
        if self.keyframes.len() == 1 || t <= self.keyframes[0].time {
            return self.keyframes[0].value;
        }
        let last = self.keyframes.last().unwrap();
        if t >= last.time {
            return last.value;
        }
        let idx = self
            .keyframes
            .partition_point(|k| k.time < t)
            .saturating_sub(1);
        let a = &self.keyframes[idx];
        let b = &self.keyframes[(idx + 1).min(self.keyframes.len() - 1)];
        let dt = (b.time - a.time).max(0.0001);
        let f = ((t - a.time) / dt).clamp(0.0, 1.0);

        match self.property {
            ChannelProperty::Rotation => {
                let qa = Quat::from_xyzw(a.value[0], a.value[1], a.value[2], a.value[3]);
                let qb = Quat::from_xyzw(b.value[0], b.value[1], b.value[2], b.value[3]);
                let q = qa.slerp(qb, f);
                [q.x, q.y, q.z, q.w]
            }
            _ => {
                let mut out = [0.0f32; 4];
                for i in 0..4 {
                    out[i] = a.value[i] + f * (b.value[i] - a.value[i]);
                }
                out
            }
        }
    }
}

pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<Channel>,
}

pub struct AnimationPlayer {
    pub current_clip: Option<usize>,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
    pub local_transforms: Vec<Mat4>,
}

impl AnimationPlayer {
    pub fn new(joint_count: usize) -> Self {
        Self {
            current_clip: None,
            time: 0.0,
            speed: 1.0,
            looping: true,
            local_transforms: vec![Mat4::IDENTITY; joint_count],
        }
    }

    pub fn play(&mut self, clip_index: usize) {
        if self.current_clip != Some(clip_index) {
            self.current_clip = Some(clip_index);
            self.time = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32, clips: &[AnimationClip], skeleton: &Skeleton) {
        let clip_idx = match self.current_clip {
            Some(i) if i < clips.len() => i,
            _ => return,
        };
        let clip = &clips[clip_idx];

        self.time += dt * self.speed;
        if self.looping && clip.duration > 0.0 {
            self.time %= clip.duration;
        } else {
            self.time = self.time.min(clip.duration);
        }

        self.local_transforms.resize(skeleton.joint_count(), Mat4::IDENTITY);
        for i in 0..skeleton.joint_count() {
            self.local_transforms[i] = skeleton.joints[i].local_bind_transform;
        }

        for channel in &clip.channels {
            let val = channel.sample(self.time);
            let ji = channel.joint_index;
            if ji >= self.local_transforms.len() {
                continue;
            }
            match channel.property {
                ChannelProperty::Translation => {
                    let t = Vec3::new(val[0], val[1], val[2]);
                    let current = self.local_transforms[ji];
                    let (s, r, _) = current.to_scale_rotation_translation();
                    self.local_transforms[ji] = Mat4::from_scale_rotation_translation(s, r, t);
                }
                ChannelProperty::Rotation => {
                    let q = Quat::from_xyzw(val[0], val[1], val[2], val[3]).normalize();
                    let current = self.local_transforms[ji];
                    let (s, _, t) = current.to_scale_rotation_translation();
                    self.local_transforms[ji] = Mat4::from_scale_rotation_translation(s, q, t);
                }
                ChannelProperty::Scale => {
                    let sv = Vec3::new(val[0], val[1], val[2]);
                    let current = self.local_transforms[ji];
                    let (_, r, t) = current.to_scale_rotation_translation();
                    self.local_transforms[ji] = Mat4::from_scale_rotation_translation(sv, r, t);
                }
            }
        }
    }
}
