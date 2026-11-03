//! Ours (the modern UI's click-to-move; one way to try the plan's open question, which stays open; not retail):
//! Ctrl + left click on the ground walks the character there. The point is where the pointer's ray meets the collision mesh; the way is an A* search on a
//! one-yalm grid over the same mesh (a step is a floor within reach of the last one with no wall between at chest
//! height), straightened where a straight walk stays on the floor. The walk itself is autorun steered at each point
//! in turn, as /follow steers it at a target, so every rule autorun keeps (events, resting, locks) still holds.

use super::ground::Ground;
use glam::Vec3;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::time::{Duration, Instant};

/// How far the pointer's ray looks for the ground, and the farthest goal from the character.
const RAY: f32 = 120.0;
const REACH: f32 = 60.0;
/// The grid step, how high a step may climb or drop (World Y points down), and the chest height a wall is looked for.
const CELL: f32 = 1.0;
const CLIMB: f32 = 0.8;
const DROP: f32 = 1.5;
const CHEST: f32 = 1.0;
/// The search gives up past this many cells.
const MAX_NODES: usize = 25_000;
/// A point counts as reached within this flat distance; no half yalm of progress in this long ends the walk.
pub const ARRIVE: f32 = 0.6;
const STUCK: Duration = Duration::from_millis(1500);

/// A walk under way: the points left, and the last place progress was measured from.
#[derive(Clone, Debug)]
pub struct ClickMove {
    pub path: Vec<Vec3>,
    pub check: (Vec3, Instant),
}

/// What a tick of the walk asks for.
#[derive(Debug, PartialEq)]
pub enum Steer {
    /// Run this (flat, unit) way.
    Toward(Vec3),
    Arrived,
    Stuck,
}

impl ClickMove {
    pub fn new(path: Vec<Vec3>, from: Vec3) -> Self {
        Self {
            path,
            check: (from, Instant::now()),
        }
    }

    /// The way to run from `here`: points reached drop off; no progress for a while gives up.
    pub fn steer(&mut self, here: Vec3) -> Steer {
        while self
            .path
            .first()
            .is_some_and(|p| flat(*p - here).length() < ARRIVE)
        {
            self.path.remove(0);
        }
        let Some(next) = self.path.first() else {
            return Steer::Arrived;
        };
        if self.check.1.elapsed() >= STUCK {
            if flat(here - self.check.0).length() < 0.5 {
                return Steer::Stuck;
            }
            self.check = (here, Instant::now());
        }
        let d = flat(*next - here);
        if d.length() < 1e-4 {
            Steer::Arrived
        } else {
            Steer::Toward(d.normalize())
        }
    }
}

fn flat(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z)
}

/// Where the ray from `eye` along `dir` first meets the collision mesh, standing on its floor.
pub fn ground_point(g: &Ground, eye: Vec3, dir: Vec3) -> Option<Vec3> {
    let t = g.raycast(eye, eye + dir.normalize() * RAY)?;
    let p = eye + dir.normalize() * t;
    // A wall hit: the floor under the point, a little back toward the eye.
    let back = p - flat(dir).normalize_or_zero() * 0.3;
    let y = g.snap(back.x, back.z, back.y)?;
    Some(Vec3::new(back.x, y, back.z))
}

/// The way's next floor from a floor at `y`: within a climb up or a drop down.
fn floor_near(g: &Ground, x: f32, z: f32, y: f32) -> Option<f32> {
    g.floor(x, z, y, CLIMB).filter(|&ny| ny - y <= DROP)
}

/// No wall between two floor points at chest height.
fn clear(g: &Ground, a: Vec3, b: Vec3) -> bool {
    let up = Vec3::new(0.0, -CHEST, 0.0);
    g.raycast(a + up, b + up).is_none()
}

/// A straight walk from `a` to `b` keeps to the floor (sampled every half yalm) with no wall in the way.
fn walkable(g: &Ground, a: Vec3, b: Vec3) -> bool {
    if !clear(g, a, b) {
        return false;
    }
    let n = (flat(b - a).length() / 0.5).ceil().max(1.0) as usize;
    let mut y = a.y;
    for k in 1..=n {
        let p = a.lerp(b, k as f32 / n as f32);
        match floor_near(g, p.x, p.z, y) {
            Some(ny) => y = ny,
            None => return false,
        }
    }
    true
}

#[derive(PartialEq)]
struct Open(f32, (i32, i32));

impl Eq for Open {}

impl Ord for Open {
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.total_cmp(&self.0)
    }
}

impl PartialOrd for Open {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// The points to walk from `from` to `to` (the goal last), or None when there is no way within reach.
pub fn path(g: &Ground, from: Vec3, to: Vec3) -> Option<Vec<Vec3>> {
    if flat(to - from).length() > REACH {
        return None;
    }
    if walkable(g, from, to) {
        return Some(vec![to]);
    }
    let at = |c: (i32, i32)| (from.x + c.0 as f32 * CELL, from.z + c.1 as f32 * CELL);
    let goal = (
        ((to.x - from.x) / CELL).round() as i32,
        ((to.z - from.z) / CELL).round() as i32,
    );
    let h = |c: (i32, i32)| (((c.0 - goal.0) as f32).hypot((c.1 - goal.1) as f32)) * CELL;
    let mut open = BinaryHeap::new();
    let mut cost: HashMap<(i32, i32), f32> = HashMap::new();
    let mut came: HashMap<(i32, i32), ((i32, i32), Vec3)> = HashMap::new();
    let start = (0, 0);
    cost.insert(start, 0.0);
    came.insert(start, (start, from));
    open.push(Open(h(start), start));
    let mut end = None;
    while let Some(Open(_, c)) = open.pop() {
        if came.len() > MAX_NODES {
            break;
        }
        let here = came[&c].1;
        if (c.0 - goal.0).abs() <= 1 && (c.1 - goal.1).abs() <= 1 && walkable(g, here, to) {
            end = Some(c);
            break;
        }
        for (dx, dz) in [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
            let n = (c.0 + dx, c.1 + dz);
            let (x, z) = at(n);
            if flat(Vec3::new(x, 0.0, z) - from).length() > REACH + 10.0 {
                continue;
            }
            let Some(y) = floor_near(g, x, z, here.y) else {
                continue;
            };
            let p = Vec3::new(x, y, z);
            if !clear(g, here, p) {
                continue;
            }
            let step = if dx != 0 && dz != 0 {
                std::f32::consts::SQRT_2
            } else {
                1.0
            } * CELL;
            let c2 = cost[&c] + step;
            if cost.get(&n).is_none_or(|&old| c2 < old) {
                cost.insert(n, c2);
                came.insert(n, (c, p));
                open.push(Open(c2 + h(n), n));
            }
        }
    }
    let mut c = end?;
    let mut points = vec![to];
    while c != start {
        let (prev, p) = came[&c];
        points.push(p);
        c = prev;
    }
    points.push(from);
    points.reverse();
    Some(straighten(g, &points))
}

/// Drops the points a straight walk can skip (the start too); the goal stays last.
fn straighten(g: &Ground, points: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < points.len() {
        let mut j = points.len() - 1;
        while j > i + 1 && !walkable(g, points[i], points[j]) {
            j -= 1;
        }
        out.push(points[j]);
        i = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steer_drops_reached_points_and_arrives() {
        let mut m = ClickMove::new(
            vec![Vec3::new(0.3, 0.0, 0.0), Vec3::new(5.0, 0.0, 0.0)],
            Vec3::ZERO,
        );
        assert_eq!(m.steer(Vec3::ZERO), Steer::Toward(Vec3::X));
        assert_eq!(m.path.len(), 1);
        assert_eq!(m.steer(Vec3::new(4.8, -3.0, 0.0)), Steer::Arrived);
    }

    #[test]
    fn steer_gives_up_without_progress() {
        let mut m = ClickMove::new(vec![Vec3::new(5.0, 0.0, 0.0)], Vec3::ZERO);
        m.check.1 = Instant::now() - STUCK;
        assert_eq!(m.steer(Vec3::new(0.1, 0.0, 0.0)), Steer::Stuck);
    }
}
