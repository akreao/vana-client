//! Retail's one task list (DAT_1047e2f0): every task, each scheduler among them, sits in one doubly linked list
//! walked once a frame (FUN_10074c30). The list's head is +0xd50, the running task +0xd4c, the walk's next +0xd5c;
//! sixteen anchor tasks (+0xd04 + 4 x bucket) split it into buckets.
//!
//! - The walk: running = head; while running: next = running's next, update running (vtable +0x20 through
//!   FUN_10075000, skipped while +0x22 holds it), running = next.
//! - A new task (base constructor FUN_10074740 -> FUN_100750b0) goes right after the running task and becomes the
//!   walk's next; with none running, at the front of bucket 9 (FUN_10074f80).
//! - A scheduler (FUN_10057270 -> FUN_10074a00 -> FUN_10075120) and a generator's task (FUN_10053370) are then moved
//!   to right before the running task, or before anchor 9 with none running. A bucket's task (FUN_100749b0 /
//!   FUN_10075220) is moved to the front of its bucket.
//! - Taking a task out (FUN_100751b0): the walk's next that pointed at it moves on to its next; the running task that
//!   was it becomes its previous.
//! - A scheduler's first update (FUN_100748c0) runs with itself as the running task and puts the old one back, but not
//!   the walk's next. So a task made in that update leaves the walk's next before the task the walk is in, which then
//!   runs again in the same frame.
//!
//! - The entity loop's task (vtable 0x1032acfc, made by menu_chdummy at 0x100142b0: FUN_10074740, then
//!   FUN_100749b0(0xd)) sits at the front of bucket 13; its update (+0x20 FUN_100106c0) is FUN_10096640, every
//!   actor's update. A scheduler that update starts is linked right before it, so in bucket 13, and takes its first
//!   update at once; a task made in that first update leaves the walk's next before the loop's task, which then
//!   runs again in the same frame.
//!
//! Only schedulers and the entity loop's task are kept for good. The other tasks (holds, drives, generators, sounds...) update in their own
//! passes; here a task an op makes is a node for the rest of the walk, so that the walk's next goes where retail's goes.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Node {
    /// A scheduler, by its `Task::uid`.
    Task(u64),
    /// Another task an op made, kept until the walk is over.
    Other(u64),
    Anchor(u8),
    /// The entity loop's task (FUN_100106c0 -> FUN_10096640).
    Entity,
}

/// How a task that is not a scheduler is linked when made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Made {
    /// Left after the running task (FUN_10074740 / FUN_10074790 and nothing more): holds, drives, ramps, sounds, dolls.
    After,
    /// Then moved: before the running task or into a bucket (generators, their elements, op 0x80's task, op 0xa6's
    /// fade). The walk's next is the running task's next.
    Moved,
}

const BUCKETS: u8 = 16;
/// The bucket a scheduler is linked before (DAT_1047e2f0 +0xd28) and a plain task into, when none is running.
const SCHED_BUCKET: u8 = 9;
/// The bucket the entity loop's task is put at the front of (FUN_100749b0(0xd) at 0x100142e6).
const ENTITY_BUCKET: u8 = 13;

#[derive(Debug)]
pub struct TaskList {
    nodes: Vec<Node>,
    running: Option<Node>,
    next: Option<Node>,
    others: u64,
    /// A walk is in (FUN_10074c30).
    pub walking: bool,
}

impl Default for TaskList {
    fn default() -> Self {
        let mut nodes: Vec<Node> = (0..BUCKETS).map(Node::Anchor).collect();
        nodes.insert(ENTITY_BUCKET as usize + 1, Node::Entity);
        Self { nodes, running: None, next: None, others: 0, walking: false }
    }
}

impl TaskList {
    fn pos(&self, n: Node) -> Option<usize> {
        self.nodes.iter().position(|&x| x == n)
    }

    fn succ(&self, n: Node) -> Option<Node> {
        self.pos(n).and_then(|i| self.nodes.get(i + 1).copied())
    }

    /// FUN_100751b0.
    pub fn unlink(&mut self, n: Node) {
        let Some(i) = self.pos(n) else { return };
        if self.running == Some(n) {
            self.running = i.checked_sub(1).map(|p| self.nodes[p]);
        }
        if self.next == Some(n) {
            self.next = self.nodes.get(i + 1).copied();
        }
        self.nodes.remove(i);
    }

    /// FUN_10074f80: at the front of bucket `b`.
    fn bucket(&mut self, n: Node, b: u8) {
        self.unlink(n);
        let at = self.pos(Node::Anchor(b.min(BUCKETS - 1))).map_or(self.nodes.len(), |i| i + 1);
        self.nodes.insert(at, n);
    }

    /// FUN_100750b0: right after the running task, and the walk's next; with none running, bucket 9.
    pub fn link_after_running(&mut self, n: Node) {
        self.unlink(n);
        match self.running.and_then(|r| self.pos(r)) {
            Some(i) => {
                self.nodes.insert(i + 1, n);
                self.next = Some(n);
            }
            None => self.bucket(n, SCHED_BUCKET),
        }
    }

    /// FUN_10075120: right before the running task; with none running, before anchor 9.
    pub fn link_before_running(&mut self, n: Node) {
        self.unlink(n);
        let at = match self.running.and_then(|r| self.pos(r)) {
            Some(i) => i,
            None => self.pos(Node::Anchor(SCHED_BUCKET)).unwrap_or(self.nodes.len()),
        };
        self.nodes.insert(at, n);
    }

    /// A scheduler's construction (FUN_10057270): base constructor, then moved before the running task.
    pub fn new_scheduler(&mut self, uid: u64) {
        let n = Node::Task(uid);
        self.link_after_running(n);
        self.link_before_running(n);
    }

    /// A copy of a scheduler (FUN_10074920): linked after the running task only.
    pub fn new_copy(&mut self, uid: u64) {
        self.link_after_running(Node::Task(uid));
    }

    /// Another task made by the running one.
    pub fn made(&mut self, how: Made) {
        if self.running.is_none() {
            // Bucket 9 or before: no walk is in, and the node would do nothing.
            return;
        }
        self.others += 1;
        let n = Node::Other(self.others);
        self.link_after_running(n);
        if how == Made::Moved {
            self.unlink(n);
        }
    }

    /// FUN_100748c0: `n` runs as the running task; returns what to put back.
    pub fn enter(&mut self, n: Node) -> Option<Node> {
        self.running.replace(n)
    }

    pub fn leave(&mut self, saved: Option<Node>) {
        self.running = saved;
    }

    /// The walk's first node (FUN_10074c30).
    pub fn head(&self) -> Option<Node> {
        self.nodes.first().copied()
    }

    /// Before a node's update in the walk: it runs, and its next is the walk's next.
    pub fn step(&mut self, n: Node) {
        self.next = self.succ(n);
        self.running = Some(n);
    }

    /// After a node's update: the walk's next, and nothing running once the walk is over.
    pub fn advance(&mut self) -> Option<Node> {
        let n = self.next.take();
        if n.is_none() {
            self.running = None;
        }
        n
    }

    /// Drops what no longer stands for a task: the other tasks and the schedulers not in `live`. A dropped task does
    /// nothing in the walk, which goes on to its next as retail's unlink would have sent it.
    pub fn purge(&mut self, live: &std::collections::HashSet<u64>) {
        let keep = |n: &Node| match *n {
            Node::Task(u) => live.contains(&u),
            Node::Other(_) => false,
            Node::Anchor(_) | Node::Entity => true,
        };
        if self.running.is_some_and(|r| !keep(&r)) || self.next.is_some_and(|n| !keep(&n)) {
            return;
        }
        self.nodes.retain(keep);
    }

    /// A scheduler not in the list yet (put in a slot directly) goes before anchor 9, as one linked with none running.
    pub fn adopt(&mut self, uid: u64) {
        if self.pos(Node::Task(uid)).is_none() {
            let at = self.pos(Node::Anchor(SCHED_BUCKET)).unwrap_or(self.nodes.len());
            self.nodes.insert(at, Node::Task(uid));
        }
    }

    /// The whole list in walk order.
    #[cfg(test)]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// The schedulers in walk order.
    #[cfg(test)]
    pub fn tasks(&self) -> impl Iterator<Item = u64> + '_ {
        self.nodes.iter().filter_map(|n| if let Node::Task(u) = n { Some(*u) } else { None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Walks the list as FUN_10074c30 does; `upd` is each node's update. The entity loop's task is left out of what
    /// is returned (walk_all keeps it).
    fn walk(l: &mut TaskList, upd: impl FnMut(&mut TaskList, Node)) -> Vec<Node> {
        walk_all(l, upd).into_iter().filter(|&n| n != Node::Entity).collect()
    }

    fn walk_all(l: &mut TaskList, mut upd: impl FnMut(&mut TaskList, Node)) -> Vec<Node> {
        let mut seen = Vec::new();
        let mut cur = l.head();
        while let Some(n) = cur {
            l.step(n);
            if !matches!(n, Node::Anchor(_)) {
                seen.push(n);
                upd(l, n);
            }
            cur = l.advance();
        }
        seen
    }

    /// The entity loop's task is the first of bucket 13 (0x100142e6); a scheduler it starts sits right before it, a
    /// later one between the two, and neither is walked in that frame.
    #[test]
    fn entity_loop_starts_go_before_it_in_bucket_13() {
        let mut l = TaskList::default();
        assert_eq!(l.pos(Node::Entity), l.pos(Node::Anchor(13)).map(|i| i + 1));
        l.new_scheduler(1);
        let mut once = true;
        let seen = walk_all(&mut l, |l, n| {
            if n == Node::Entity && std::mem::take(&mut once) {
                for u in [2, 3] {
                    l.new_scheduler(u);
                    let s = l.enter(Node::Task(u));
                    l.leave(s);
                }
            }
        });
        assert_eq!(seen, vec![Node::Task(1), Node::Entity]);
        let i = |n| l.pos(n).unwrap();
        assert!(i(Node::Task(1)) < i(Node::Anchor(9)));
        assert!(i(Node::Anchor(13)) < i(Node::Task(2)));
        assert_eq!([i(Node::Task(2)) + 1, i(Node::Task(3)) + 1], [i(Node::Task(3)), i(Node::Entity)]);
    }

    /// A start in the entity loop whose first update makes a task runs the loop's task again; a later start that
    /// makes none puts the walk's next back after it (FUN_10075120's unlink of the new scheduler).
    #[test]
    fn entity_loop_runs_again_after_a_start_that_makes_a_task() {
        for (how, later, runs) in
            [(Made::After, false, 2), (Made::Moved, false, 2), (Made::After, true, 1), (Made::Moved, true, 1)]
        {
            let mut l = TaskList::default();
            let mut once = true;
            let seen = walk_all(&mut l, |l, n| {
                if n == Node::Entity && std::mem::take(&mut once) {
                    l.new_scheduler(1);
                    let s = l.enter(Node::Task(1));
                    l.made(how);
                    l.leave(s);
                    if later {
                        l.new_scheduler(2);
                        let s = l.enter(Node::Task(2));
                        l.leave(s);
                    }
                }
            });
            assert_eq!(seen.iter().filter(|&&n| n == Node::Entity).count(), runs, "{how:?} later {later}");
            // The started scheduler itself is not walked in that frame.
            assert!(!seen.contains(&Node::Task(1)));
        }
    }

    #[test]
    fn schedulers_outside_a_walk_go_before_bucket_9_in_order() {
        let mut l = TaskList::default();
        l.new_scheduler(1);
        l.new_scheduler(2);
        let i = |n| l.pos(n).unwrap();
        assert!(i(Node::Anchor(8)) < i(Node::Task(1)));
        assert!(i(Node::Task(1)) < i(Node::Task(2)));
        assert!(i(Node::Task(2)) < i(Node::Anchor(9)));
    }

    /// A child started by T is linked before T: not walked again this frame, and in front of T from then on.
    #[test]
    fn child_with_nothing_made_runs_no_one_twice() {
        let mut l = TaskList::default();
        l.new_scheduler(1);
        let seen = walk(&mut l, |l, n| {
            if n == Node::Task(1) && l.pos(Node::Task(2)).is_none() {
                l.new_scheduler(2);
                let s = l.enter(Node::Task(2));
                l.leave(s);
            }
        });
        assert_eq!(seen, vec![Node::Task(1)]);
        assert_eq!(l.tasks().collect::<Vec<_>>(), vec![2, 1]);
    }

    /// The row-172 case: the child's first update makes a task, so the walk's next is that task, whose next is T.
    #[test]
    fn child_that_makes_a_task_runs_its_parent_twice() {
        for how in [Made::After, Made::Moved] {
            let mut l = TaskList::default();
            l.new_scheduler(1);
            l.new_scheduler(3);
            let mut once = true;
            let seen = walk(&mut l, |l, n| {
                if n == Node::Task(1) && std::mem::take(&mut once) {
                    l.new_scheduler(2);
                    let s = l.enter(Node::Task(2));
                    l.made(how);
                    l.leave(s);
                }
            });
            let tasks: Vec<_> = seen.into_iter().filter(|n| matches!(n, Node::Task(_))).collect();
            assert_eq!(tasks, vec![Node::Task(1), Node::Task(1), Node::Task(3)], "{how:?}");
        }
    }

    /// T's own task made after the child's leaves the walk's next after T: no second run.
    #[test]
    fn parent_making_a_task_last_runs_once() {
        let mut l = TaskList::default();
        l.new_scheduler(1);
        l.new_scheduler(3);
        let mut once = true;
        let seen = walk(&mut l, |l, n| {
            if n == Node::Task(1) && std::mem::take(&mut once) {
                l.new_scheduler(2);
                let s = l.enter(Node::Task(2));
                l.made(Made::After);
                l.leave(s);
                l.made(Made::Moved);
            }
        });
        let tasks: Vec<_> = seen.into_iter().filter(|n| matches!(n, Node::Task(_))).collect();
        assert_eq!(tasks, vec![Node::Task(1), Node::Task(3)]);
    }

    /// Nested: T starts S, S's first update starts S2 whose first update makes a plain task. The walk goes on at
    /// that task, then S (a second update after its first), then T again.
    #[test]
    fn nested_child_runs_both_parents_again() {
        let mut l = TaskList::default();
        l.new_scheduler(1);
        let mut once = true;
        let seen = walk(&mut l, |l, n| {
            if n == Node::Task(1) && std::mem::take(&mut once) {
                l.new_scheduler(2);
                let s = l.enter(Node::Task(2));
                l.new_scheduler(3);
                let s2 = l.enter(Node::Task(3));
                l.made(Made::After);
                l.leave(s2);
                l.leave(s);
            }
        });
        let tasks: Vec<_> = seen.into_iter().filter(|n| matches!(n, Node::Task(_))).collect();
        assert_eq!(tasks, vec![Node::Task(1), Node::Task(2), Node::Task(1)]);
    }

    /// A copy (FUN_10074920) is linked after the running task and walked next.
    #[test]
    fn copy_runs_right_after_its_original() {
        let mut l = TaskList::default();
        l.new_scheduler(1);
        l.new_scheduler(3);
        let mut once = true;
        let seen = walk(&mut l, |l, n| {
            if n == Node::Task(1) && std::mem::take(&mut once) {
                l.new_copy(2);
            }
        });
        assert_eq!(seen, vec![Node::Task(1), Node::Task(2), Node::Task(3)]);
    }

    /// Taking out the walk's next moves it on (FUN_100751b0).
    #[test]
    fn unlinking_the_next_moves_it_on() {
        let mut l = TaskList::default();
        for u in 1..=3 {
            l.new_scheduler(u);
        }
        let seen = walk(&mut l, |l, n| {
            if n == Node::Task(1) {
                l.unlink(Node::Task(2));
            }
        });
        assert_eq!(seen, vec![Node::Task(1), Node::Task(3)]);
    }
}
