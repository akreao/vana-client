//! Retail's point-light slot leak (FUN_1017c9c0, FUN_1017b510). Each scenery holder starts with the four point-light
//! slots off and their ref caches 0; a drawn placement whose ref in a slot differs from the cache sets the cache and
//! then turns the slot off (ref 0) or sets it to the ref's light, but leaves it as it is for a key starting with 'c'.
//! Such a placement is lit by the last drawn placement before it, in draw order, whose ref there is 0 or a valid key
//! (off when there is none). Draw order: the camera list (else every placement), those with +0xf (file +0x46 bit 2)
//! queued after the rest; drawn = within its distance, inside the frustum (FUN_10181bb0) and with a mesh at that
//! distance. Ours replays that on the CPU for the few placements with such refs and writes their instances' slots.

use super::Instance;
use glam::{Mat4, Vec3, Vec4};
use std::collections::HashMap;

/// A placement's ref in one slot as the scenery pass treats it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ref {
    /// Ref 0: the slot off.
    Off,
    /// A valid key: its table slot + 1.
    Set(u32),
    /// A 'c' key: the slot left as it is.
    Keep,
}

/// What the replay needs of a geometry, built with it when any scenery placement has a Keep ref.
#[derive(Clone, Default)]
pub struct LeakData {
    /// Per placement (scene index): its refs, whether it is queued after the rest, and its scenery instances.
    pub places: Vec<([Ref; 4], bool, Vec<u32>)>,
    /// Per instance: as built, and its culling bounds (cull.rs: centre, own distance², half extents, rule).
    pub insts: Vec<(Instance, [f32; 8])>,
    /// The placements with a Keep ref.
    pub leakers: Vec<u32>,
}

impl LeakData {
    pub fn has_keep(refs: &[Ref; 4]) -> bool {
        refs.contains(&Ref::Keep)
    }
}

/// The per-frame view the replay tests against: the view-projection, the eye, the draw distance D (0 none), the
/// Clipping factor K, the voyage matrix.
pub struct View {
    pub view_proj: Mat4,
    pub eye: Vec3,
    pub far: f32,
    pub k: f32,
    pub voyage: Mat4,
}

impl View {
    /// cs_cull_vis without the camera-list mask (the replay walks the list itself).
    fn visible(&self, inst: &Instance, b: &[f32; 8]) -> bool {
        let mut model = Mat4::from_cols_array_2d(&inst.model);
        let voyaged = inst.band[3] != 0.0;
        let m3 = model.w_axis;
        let origin = if voyaged { (self.voyage * m3).truncate() } else { m3.truncate() };
        let d2 = (self.eye - origin).length_squared();
        let own = self.k * b[3];
        let zone = if self.far > 0.0 { self.far * self.far } else { f32::MAX };
        let limit = if b[7] > 1.5 {
            own.min(zone)
        } else if b[7] > 0.5 {
            own
        } else {
            zone
        };
        if d2 > limit {
            return false;
        }
        let band = inst.band;
        if band[1] > 0.0 && !(d2 > band[0] && d2 <= band[1]) {
            return false;
        }
        if voyaged {
            model = self.voyage * model;
        }
        let centre = model.transform_point3(Vec3::new(b[0], b[1], b[2]));
        let axes = [model.x_axis.truncate() * b[4], model.y_axis.truncate() * b[5], model.z_axis.truncate() * b[6]];
        let r = [self.view_proj.row(0), self.view_proj.row(1), self.view_proj.row(2), self.view_proj.row(3)];
        let depth = if self.far > 0.0 { Vec4::new(0.0, 0.0, 0.0, self.far) - r[3] } else { Vec4::W };
        [r[3] + r[0], r[3] - r[0], r[3] + r[1], r[3] - r[1], r[2], r[3] - r[2], depth].iter().all(|p| {
            let n = p.truncate();
            let reach: f32 = axes.iter().map(|a| n.dot(*a).abs()).sum();
            n.dot(centre) + p.w >= -reach
        })
    }
}

/// The replay's state on a renderer: its data, the draw order for the camera list in force and each leaker's place
/// in it, and the slots last written per instance.
pub struct LeakState {
    pub data: LeakData,
    order: Vec<u32>,
    at: HashMap<u32, usize>,
    pub written: HashMap<u32, [u32; 4]>,
}

impl LeakState {
    pub fn new(data: LeakData) -> Self {
        let mut s = Self { data, order: Vec::new(), at: HashMap::new(), written: HashMap::new() };
        s.set_list(None);
        s
    }

    /// FUN_1017b510's order for a camera list (placement indices), or every placement without one: the scenery
    /// placements in it, then those queued (+0xf) in the same order.
    pub fn set_list(&mut self, list: Option<&[u32]>) {
        let all: Vec<u32>;
        let list = match list {
            Some(l) => l,
            None => {
                all = (0..self.data.places.len() as u32).collect();
                &all
            }
        };
        let scenery = |p: &u32| self.data.places.get(*p as usize).is_some_and(|x| !x.2.is_empty());
        let deferred = |p: &u32| self.data.places[*p as usize].1;
        self.order = list.iter().filter(|p| scenery(p) && !deferred(p)).copied().collect();
        self.order.extend(list.iter().filter(|p| scenery(p) && deferred(p)).copied());
        self.at = self
            .order
            .iter()
            .enumerate()
            .filter(|(_, p)| self.data.leakers.contains(p))
            .map(|(i, p)| (*p, i))
            .collect();
    }

    /// Each leaker's instances' slots this frame (Keep slots resolved; a leaker not in the order keeps them off).
    /// `hidden(i)`: instance i is not drawn (a stand-in for the shown sub-area).
    pub fn resolve(&self, view: &View, hidden: impl Fn(u32) -> bool) -> HashMap<u32, [u32; 4]> {
        let mut drawn: HashMap<u32, bool> = HashMap::new();
        let mut is_drawn = |p: u32| -> bool {
            *drawn.entry(p).or_insert_with(|| {
                self.data.places[p as usize].2.iter().any(|&i| {
                    let (inst, b) = &self.data.insts[i as usize];
                    !hidden(i) && view.visible(inst, b)
                })
            })
        };
        let mut out = HashMap::new();
        for &p in &self.data.leakers {
            let (refs, _, insts) = &self.data.places[p as usize];
            let mut slots = [0u32; 4];
            for k in 0..4 {
                slots[k] = match refs[k] {
                    Ref::Off => 0,
                    Ref::Set(s) => s,
                    Ref::Keep => {
                        let Some(&at) = self.at.get(&p) else { continue };
                        self.order[..at]
                            .iter()
                            .rev()
                            .filter(|&&q| self.data.places[q as usize].0[k] != Ref::Keep)
                            .find(|&&q| is_drawn(q))
                            .map_or(0, |&q| match self.data.places[q as usize].0[k] {
                                Ref::Set(s) => s,
                                _ => 0,
                            })
                    }
                };
            }
            for &i in insts {
                out.insert(i, slots);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inst(x: f32) -> Instance {
        Instance {
            model: Mat4::from_translation(Vec3::new(x, 0.0, 10.0)).to_cols_array_2d(),
            lights: [0; 4],
            area: [0; 4],
            band: [0.0; 4],
        }
    }

    /// A 'c' ref takes the last drawn ref before it that is 0 or valid; an undrawn one in between is passed over; with
    /// none before it, off.
    #[test]
    fn keep_takes_last_drawn() {
        let b = [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];
        let k = Ref::Keep;
        let data = LeakData {
            places: vec![
                ([Ref::Set(3), Ref::Off, Ref::Off, Ref::Off], false, vec![0]),
                ([Ref::Set(5), Ref::Off, Ref::Off, Ref::Off], false, vec![1]),
                ([k, k, Ref::Off, Ref::Off], false, vec![2]),
                ([k, Ref::Off, Ref::Off, Ref::Off], true, vec![3]),
                ([Ref::Set(7), Ref::Off, Ref::Off, Ref::Off], false, vec![4]),
            ],
            insts: vec![(inst(0.0), b), (inst(500.0), b), (inst(0.0), b), (inst(0.0), b), (inst(0.0), b)],
            leakers: vec![2, 3],
        };
        let s = LeakState::new(data);
        let view = View {
            view_proj: glam::camera::rh::proj::directx::perspective(1.0, 1.0, 0.1, 1000.0)
                * glam::camera::rh::view::look_at_mat4(Vec3::ZERO, Vec3::Z, Vec3::Y),
            eye: Vec3::ZERO,
            far: 0.0,
            k: 1.0,
            voyage: Mat4::IDENTITY,
        };
        let r = s.resolve(&view, |_| false);
        // Placement 1 is off to the side (not drawn): 2 takes 0's light; the queued 3 comes after 4.
        assert_eq!(r[&2], [3, 0, 0, 0]);
        assert_eq!(r[&3], [7, 0, 0, 0]);
        // Hiding 0 leaves 2 with nothing before it.
        assert_eq!(s.resolve(&view, |i| i == 0)[&2], [0, 0, 0, 0]);
    }
}
