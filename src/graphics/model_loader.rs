use std::path::Path;

use glam::{Mat4, Quat, Vec3};

use crate::animation::animation::{
    AnimationClip, Channel, ChannelProperty, Joint, Keyframe, Skeleton,
};
use crate::graphics::skinning::SkinVertex;

pub struct LoadedModel {
    pub vertices: Vec<SkinVertex>,
    pub indices: Vec<u32>,
    pub skeleton: Skeleton,
    pub clips: Vec<AnimationClip>,
}

pub fn load_glb(path: &Path) -> Result<LoadedModel, String> {
    let (document, buffers, _images) =
        gltf::import(path).map_err(|e| format!("glTF load error: {e}"))?;

    let mesh = document
        .meshes()
        .next()
        .ok_or("no mesh in glTF")?;

    let primitive = mesh
        .primitives()
        .next()
        .ok_or("no primitive in mesh")?;

    let reader = primitive.reader(|b| Some(&buffers[b.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or("no positions")?
        .collect();

    let normals: Vec<[f32; 3]> = reader
        .read_normals()
        .map(|n| n.collect())
        .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

    let joints_data: Vec<[u16; 4]> = reader
        .read_joints(0)
        .map(|j| j.into_u16().collect())
        .unwrap_or_else(|| vec![[0; 4]; positions.len()]);

    let weights: Vec<[f32; 4]> = reader
        .read_weights(0)
        .map(|w| w.into_f32().collect())
        .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]);

    let vertices: Vec<SkinVertex> = positions
        .iter()
        .enumerate()
        .map(|(i, p)| SkinVertex {
            position: *p,
            normal: normals[i],
            joints: [
                joints_data[i][0] as u32,
                joints_data[i][1] as u32,
                joints_data[i][2] as u32,
                joints_data[i][3] as u32,
            ],
            weights: weights[i],
        })
        .collect();

    let indices: Vec<u32> = reader
        .read_indices()
        .map(|idx| idx.into_u32().collect())
        .unwrap_or_else(|| (0..vertices.len() as u32).collect());

    let skin = document.skins().next();
    let mut skeleton = Skeleton::new();

    if let Some(ref skin) = skin {
        let ibm_reader = skin.reader(|b| Some(&buffers[b.index()]));
        let ibms: Vec<Mat4> = ibm_reader
            .read_inverse_bind_matrices()
            .map(|iter| {
                iter.map(|m| Mat4::from_cols_array_2d(&m))
                    .collect()
            })
            .unwrap_or_default();
        skeleton.inverse_bind_matrices = ibms;

        let joint_node_indices: Vec<usize> =
            skin.joints().map(|j| j.index()).collect();

        let mut parent_map: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        fn walk_parents(
            node: &gltf::Node,
            parent_map: &mut std::collections::HashMap<usize, usize>,
        ) {
            for child in node.children() {
                parent_map.insert(child.index(), node.index());
                walk_parents(&child, parent_map);
            }
        }
        for scene in document.scenes() {
            for node in scene.nodes() {
                walk_parents(&node, &mut parent_map);
            }
        }

        for node in skin.joints() {
            let (t, r, s) = node.transform().decomposed();
            let local = Mat4::from_scale_rotation_translation(
                Vec3::from(s),
                Quat::from_array(r),
                Vec3::from(t),
            );
            let parent = parent_map
                .get(&node.index())
                .and_then(|&parent_node_idx| {
                    joint_node_indices
                        .iter()
                        .position(|&ji| ji == parent_node_idx)
                });
            skeleton.joints.push(Joint {
                name: node.name().unwrap_or("joint").to_string(),
                parent,
                local_bind_transform: local,
            });
        }
    }

    let mut clips = Vec::new();
    for anim in document.animations() {
        let mut channels = Vec::new();
        for channel in anim.channels() {
            let reader = channel.reader(|b| Some(&buffers[b.index()]));
            let times: Vec<f32> = reader.read_inputs().map(|i| i.collect()).unwrap_or_default();
            let target = channel.target();
            let joint_index = if let Some(ref skin) = skin {
                skin.joints()
                    .position(|j| j.index() == target.node().index())
                    .unwrap_or(0)
            } else {
                0
            };

            let (property, values) = match reader.read_outputs() {
                Some(gltf::animation::util::ReadOutputs::Translations(t)) => (
                    ChannelProperty::Translation,
                    t.map(|v| [v[0], v[1], v[2], 0.0]).collect::<Vec<_>>(),
                ),
                Some(gltf::animation::util::ReadOutputs::Rotations(r)) => (
                    ChannelProperty::Rotation,
                    r.into_f32()
                        .map(|v| [v[0], v[1], v[2], v[3]])
                        .collect::<Vec<_>>(),
                ),
                Some(gltf::animation::util::ReadOutputs::Scales(s)) => (
                    ChannelProperty::Scale,
                    s.map(|v| [v[0], v[1], v[2], 1.0]).collect::<Vec<_>>(),
                ),
                _ => continue,
            };

            let keyframes: Vec<Keyframe> = times
                .iter()
                .zip(values.iter())
                .map(|(&time, &value)| Keyframe { time, value })
                .collect();

            channels.push(Channel {
                joint_index,
                property,
                keyframes,
            });
        }

        let duration = anim
            .channels()
            .flat_map(|c| {
                c.reader(|b| Some(&buffers[b.index()]))
                    .read_inputs()
                    .map(|i| i.last().unwrap_or(0.0))
            })
            .fold(0.0f32, f32::max);

        clips.push(AnimationClip {
            name: anim.name().unwrap_or("anim").to_string(),
            duration,
            channels,
        });
    }

    Ok(LoadedModel {
        vertices,
        indices,
        skeleton,
        clips,
    })
}
