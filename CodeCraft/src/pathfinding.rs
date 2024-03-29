//! Compute a shortest path (or all shorted paths) using the [A* search
//! algorithm](https://en.wikipedia.org/wiki/A*_search_algorithm).
use super::indexmap::map::Entry::{Occupied, Vacant};
use super::indexmap::IndexMap;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::hash::Hash;
use std::ops::{Add, Sub};
use std::usize;

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Unfold<St, F> {
    f: F,
    /// Internal state that will be passed to the closure on the next iteration
    pub state: St,
}

impl<A, St, F> Iterator for Unfold<St, F>
where
    F: FnMut(&mut St) -> Option<A>,
{
    type Item = A;

    #[inline]
    fn next(&mut self) -> Option<A> {
        (self.f)(&mut self.state)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // no possible known bounds at this point
        (0, None)
    }
}

fn unfold<A, St, F>(initial_state: St, f: F) -> Unfold<St, F>
where
    F: FnMut(&mut St) -> Option<A>,
{
    Unfold {
        f,
        state: initial_state,
    }
}
#[inline]
#[allow(dead_code)]
pub fn absdiff<T>(x: T, y: T) -> T
where
    T: Sub<Output = T> + PartialOrd,
{
    if x < y {
        y - x
    } else {
        x - y
    }
}

fn reverse_path<N, V, F>(parents: &IndexMap<N, V>, mut parent: F, start: usize) -> Vec<N>
where
    N: Eq + Hash + Clone,
    F: FnMut(&V) -> usize,
{
    let path = unfold(start, |i| {
        parents.get_index(*i).map(|(node, value)| {
            *i = parent(value);
            node
        })
    })
    .collect::<Vec<&N>>();

    path.into_iter().rev().cloned().collect()
}

pub trait Zero: Sized + Add<Self, Output = Self> {
    /// Returns the additive identity element of `Self`, `0`.
    ///
    /// # Laws
    ///
    /// ```{.text}
    /// a + 0 = a       ∀ a ∈ Self
    /// 0 + a = a       ∀ a ∈ Self
    /// ```
    ///
    /// # Purity
    ///
    /// This function should return the same result at all times regardless of
    /// external mutable state, for example values stored in TLS or in
    /// `static mut`s.
    // FIXME (#5527): This should be an associated constant
    fn zero() -> Self;

    /// Returns `true` if `self` is equal to the additive identity.
    fn is_zero(&self) -> bool;
}

impl Zero for u64 {
    #[inline]
    fn zero() -> u64 {
        0u64
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0u64
    }
}

impl Zero for u32 {
    #[inline]
    fn zero() -> u32 {
        0u32
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0u32
    }
}

impl Zero for i32 {
    #[inline]
    fn zero() -> i32 {
        0i32
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0i32
    }
}

/// Compute a shortest path using the [A* search
/// algorithm](https://en.wikipedia.org/wiki/A*_search_algorithm).
///
/// The shortest path starting from `start` up to a node for which `success` returns `true` is
/// computed and returned along with its total cost, in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node, along with the cost for moving
/// from the node to the successor.
/// - `heuristic` returns an approximation of the cost from a given node to the goal. The
/// approximation must not be greater than the real cost, or a wrong shortest path may be returned.
/// - `success` checks whether the goal has been reached. It is not a node as some problems require
/// a dynamic solution instead of a fixed node.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// The returned path comprises both the start and end node.
///
/// # Example
///
/// We will search the shortest path on a chess board to go from (1, 1) to (4, 6) doing only knight
/// moves.
///
/// The first version uses an explicit type `Pos` on which the required traits are derived.
///
/// ```
/// use pathfinding::prelude::{absdiff, astar};
///
/// #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// struct Pos(i32, i32);
///
/// impl Pos {
///   fn distance(&self, other: &Pos) -> u32 {
///     (absdiff(self.0, other.0) + absdiff(self.1, other.1)) as u32
///   }
///
///   fn successors(&self) -> Vec<(Pos, u32)> {
///     let &Pos(x, y) = self;
///     vec![Pos(x+1,y+2), Pos(x+1,y-2), Pos(x-1,y+2), Pos(x-1,y-2),
///          Pos(x+2,y+1), Pos(x+2,y-1), Pos(x-2,y+1), Pos(x-2,y-1)]
///          .into_iter().map(|p| (p, 1)).collect()
///   }
/// }
///
/// static GOAL: Pos = Pos(4, 6);
/// let result = astar(&Pos(1, 1), |p| p.successors(), |p| p.distance(&GOAL) / 3,
///                    |p| *p == GOAL);
/// assert_eq!(result.expect("no path found").1, 4);
/// ```
///
/// The second version does not declare a `Pos` type, makes use of more closures,
/// and is thus shorter.
///
/// ```
/// use pathfinding::prelude::{absdiff, astar};
///
/// static GOAL: (i32, i32) = (4, 6);
/// let result = astar(&(1, 1),
///                    |&(x, y)| vec![(x+1,y+2), (x+1,y-2), (x-1,y+2), (x-1,y-2),
///                                   (x+2,y+1), (x+2,y-1), (x-2,y+1), (x-2,y-1)]
///                               .into_iter().map(|p| (p, 1)),
///                    |&(x, y)| absdiff(x, GOAL.0) + absdiff(y, GOAL.1),
///                    |&p| p == GOAL);
/// assert_eq!(result.expect("no path found").1, 4);
/// ```
#[allow(dead_code)]
pub fn astar<N, C, FN, IN, FH, FS>(
    start: &N,
    mut successors: FN,
    mut heuristic: FH,
    mut success: FS,
) -> Option<(Vec<N>, C)>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let mut to_see = BinaryHeap::new();
    to_see.push(SmallestCostHolder {
        estimated_cost: heuristic(start),
        cost: Zero::zero(),
        index: 0,
    });
    let mut parents: IndexMap<N, (usize, C)> = IndexMap::new();
    parents.insert(start.clone(), (usize::max_value(), Zero::zero()));
    while let Some(SmallestCostHolder { cost, index, .. }) = to_see.pop() {
        let successors = {
            let (node, &(_, c)) = parents.get_index(index).unwrap();
            if success(node) {
                let path = reverse_path(&parents, |&(p, _)| p, index);
                return Some((path, cost));
            }
            // We may have inserted a node several time into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            if cost > c {
                continue;
            }
            successors(node)
        };
        for (successor, move_cost) in successors {
            let new_cost = cost + move_cost;
            let h; // heuristic(&successor)
            let n; // index for successor
            match parents.entry(successor) {
                Vacant(e) => {
                    h = heuristic(e.key());
                    n = e.index();
                    e.insert((index, new_cost));
                }
                Occupied(mut e) => {
                    if e.get().1 > new_cost {
                        h = heuristic(e.key());
                        n = e.index();
                        e.insert((index, new_cost));
                    } else {
                        continue;
                    }
                }
            }

            to_see.push(SmallestCostHolder {
                estimated_cost: new_cost + h,
                cost: new_cost,
                index: n,
            });
        }
    }
    None
}

/// Compute all shortest paths using the [A* search
/// algorithm](https://en.wikipedia.org/wiki/A*_search_algorithm). Whereas `astar`
/// (non-deterministic-ally) returns a single shortest path, `astar_bag` returns all shortest paths
/// (in a non-deterministic order).
///
/// The shortest paths starting from `start` up to a node for which `success` returns `true` are
/// computed and returned in an iterator along with the cost (which, by definition, is the same for
/// each shortest path), wrapped in a `Some`. If no paths are found, `None` is returned.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node, along with the cost for moving
/// from the node to the successor.
/// - `heuristic` returns an approximation of the cost from a given node to the goal. The
/// approximation must not be greater than the real cost, or a wrong shortest path may be returned.
/// - `success` checks whether the goal has been reached. It is not a node as some problems require
/// a dynamic solution instead of a fixed node.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// Each path comprises both the start and an end node. Note that while every path shares the same
/// start node, different paths may have different end nodes.
#[allow(dead_code)]
pub fn astar_bag<N, C, FN, IN, FH, FS>(
    start: &N,
    mut successors: FN,
    mut heuristic: FH,
    mut success: FS,
) -> Option<(AstarSolution<N>, C)>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    let mut to_see = BinaryHeap::new();
    let mut min_cost = None;
    let mut sinks = HashSet::new();
    to_see.push(SmallestCostHolder {
        estimated_cost: heuristic(start),
        cost: Zero::zero(),
        index: 0,
    });
    let mut parents: IndexMap<N, (HashSet<usize>, C)> = IndexMap::new();
    parents.insert(start.clone(), (HashSet::new(), Zero::zero()));
    while let Some(SmallestCostHolder {
        cost,
        index,
        estimated_cost,
        ..
    }) = to_see.pop()
    {
        if let Some(min_cost) = min_cost {
            if estimated_cost > min_cost {
                break;
            }
        }
        let successors = {
            let (node, &(_, c)) = parents.get_index(index).unwrap();
            if success(node) {
                min_cost = Some(cost);
                sinks.insert(index);
            }
            // We may have inserted a node several time into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            if cost > c {
                continue;
            }
            successors(node)
        };
        for (successor, move_cost) in successors {
            let new_cost = cost + move_cost;
            let h; // heuristic(&successor)
            let n; // index for successor
            match parents.entry(successor) {
                Vacant(e) => {
                    h = heuristic(e.key());
                    n = e.index();
                    let mut p = HashSet::new();
                    p.insert(index);
                    e.insert((p, new_cost));
                }
                Occupied(mut e) => {
                    if e.get().1 > new_cost {
                        h = heuristic(e.key());
                        n = e.index();
                        let s = e.get_mut();
                        s.0.clear();
                        s.0.insert(index);
                        s.1 = new_cost;
                    } else {
                        if e.get().1 == new_cost {
                            // New parent with an identical cost, this is not
                            // considered as an insertion.
                            e.get_mut().0.insert(index);
                        }
                        continue;
                    }
                }
            }

            to_see.push(SmallestCostHolder {
                estimated_cost: new_cost + h,
                cost: new_cost,
                index: n,
            });
        }
    }

    min_cost.map(|cost| {
        let parents = parents
            .into_iter()
            .map(|(k, (ps, _))| (k, ps.into_iter().collect()))
            .collect();
        (
            AstarSolution {
                sinks: sinks.into_iter().collect(),
                parents,
                current: vec![],
                terminated: false,
            },
            cost,
        )
    })
}

/// Compute all shortest paths using the [A* search
/// algorithm](https://en.wikipedia.org/wiki/A*_search_algorithm). Whereas `astar`
/// (non-deterministic-ally) returns a single shortest path, `astar_bag` returns all shortest paths
/// (in a non-deterministic order).
///
/// This is a utility function which collects the results of the `astar_bag` function into a
/// vector. Most of the time, it is more appropriate to use `astar_bag` directly.
///
/// ### Warning
///
/// The number of results with the same value might be very large in some graphs. Use with caution.
#[allow(dead_code)]
pub fn astar_bag_collect<N, C, FN, IN, FH, FS>(
    start: &N,
    successors: FN,
    heuristic: FH,
    success: FS,
) -> Option<(Vec<Vec<N>>, C)>
where
    N: Eq + Hash + Clone,
    C: Zero + Ord + Copy,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = (N, C)>,
    FH: FnMut(&N) -> C,
    FS: FnMut(&N) -> bool,
{
    astar_bag(start, successors, heuristic, success)
        .map(|(solutions, cost)| (solutions.collect(), cost))
}

#[allow(dead_code)]
struct SmallestCostHolder<K> {
    estimated_cost: K,
    cost: K,
    index: usize,
}

impl<K: PartialEq> PartialEq for SmallestCostHolder<K> {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost.eq(&other.estimated_cost) && self.cost.eq(&other.cost)
    }
}

impl<K: PartialEq> Eq for SmallestCostHolder<K> {}

impl<K: Ord> PartialOrd for SmallestCostHolder<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord> Ord for SmallestCostHolder<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.estimated_cost.cmp(&self.estimated_cost) {
            Ordering::Equal => self.cost.cmp(&other.cost),
            s => s,
        }
    }
}

/// Iterator structure created by the `astar_bag` function.
#[derive(Clone)]
pub struct AstarSolution<N> {
    sinks: Vec<usize>,
    parents: Vec<(N, Vec<usize>)>,
    current: Vec<Vec<usize>>,
    terminated: bool,
}

impl<N: Clone + Eq + Hash> AstarSolution<N> {
    fn complete(&mut self) {
        loop {
            let ps = match self.current.last() {
                None => self.sinks.clone(),
                Some(last) => {
                    let &top = last.last().unwrap();
                    self.parents(top).clone()
                }
            };
            if ps.is_empty() {
                break;
            }
            self.current.push(ps);
        }
    }

    fn next_vec(&mut self) {
        while self.current.last().map(Vec::len) == Some(1) {
            self.current.pop();
        }
        self.current.last_mut().map(Vec::pop);
    }

    fn node(&self, i: usize) -> &N {
        &self.parents[i].0
    }

    fn parents(&self, i: usize) -> &Vec<usize> {
        &self.parents[i].1
    }
}

impl<N: Clone + Eq + Hash> Iterator for AstarSolution<N> {
    type Item = Vec<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.terminated {
            return None;
        }
        self.complete();
        let path = self
            .current
            .iter()
            .rev()
            .map(|v| v.last().cloned().unwrap())
            .map(|i| self.node(i).clone())
            .collect::<Vec<_>>();
        self.next_vec();
        self.terminated = self.current.is_empty();
        Some(path)
    }
}

// BFS

/// Compute a shortest path using the [breadth-first search
/// algorithm](https://en.wikipedia.org/wiki/Breadth-first_search).
///
/// The shortest path starting from `start` up to a node for which `success` returns `true` is
/// computed and returned in a `Some`. If no path can be found, `None`
/// is returned instead.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
/// - `success` checks whether the goal has been reached. It is not a node as some problems require
/// a dynamic solution instead of a fixed node.
///
/// A node will never be included twice in the path as determined by the `Eq` relationship.
///
/// The returned path comprises both the start and end node.
///
/// # Example
///
/// We will search the shortest path on a chess board to go from (1, 1) to (4, 6) doing only knight
/// moves.
///
/// The first version uses an explicit type `Pos` on which the required traits are derived.
///
/// ```
/// use pathfinding::prelude::bfs;
///
/// #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// struct Pos(i32, i32);
///
/// impl Pos {
///   fn successors(&self) -> Vec<Pos> {
///     let &Pos(x, y) = self;
///     vec![Pos(x+1,y+2), Pos(x+1,y-2), Pos(x-1,y+2), Pos(x-1,y-2),
///          Pos(x+2,y+1), Pos(x+2,y-1), Pos(x-2,y+1), Pos(x-2,y-1)]
///   }
/// }
///
/// static GOAL: Pos = Pos(4, 6);
/// let result = bfs(&Pos(1, 1), |p| p.successors(), |p| *p == GOAL);
/// assert_eq!(result.expect("no path found").len(), 5);
/// ```
///
/// The second version does not declare a `Pos` type, makes use of more closures,
/// and is thus shorter.
///
/// ```
/// use pathfinding::prelude::bfs;
///
/// static GOAL: (i32, i32) = (4, 6);
/// let result = bfs(&(1, 1),
///                  |&(x, y)| vec![(x+1,y+2), (x+1,y-2), (x-1,y+2), (x-1,y-2),
///                                 (x+2,y+1), (x+2,y-1), (x-2,y+1), (x-2,y-1)],
///                  |&p| p == GOAL);
/// assert_eq!(result.expect("no path found").len(), 5);
/// ```
pub fn bfs<N, FN, IN, FS>(start: &N, successors: FN, success: FS) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    bfs_core(start, successors, success, true)
}

fn bfs_core<N, FN, IN, FS>(
    start: &N,
    mut successors: FN,
    mut success: FS,
    check_first: bool,
) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    if check_first && success(start) {
        return Some(vec![start.clone()]);
    }
    let mut to_see = VecDeque::new();
    let mut parents: IndexMap<N, usize> = IndexMap::new();
    to_see.push_back(0);
    parents.insert(start.clone(), usize::max_value());
    while let Some(i) = to_see.pop_front() {
        let node = parents.get_index(i).unwrap().0;
        for successor in successors(node) {
            if success(&successor) {
                let mut path = reverse_path(&parents, |&p| p, i);
                path.push(successor);
                return Some(path);
            }
            if let Vacant(e) = parents.entry(successor) {
                to_see.push_back(e.index());
                e.insert(i);
            }
        }
    }
    None
}

/// Return one of the shortest loop from start to start if it exists, `None` otherwise.
///
/// - `start` is the starting node.
/// - `successors` returns a list of successors for a given node.
///
/// Except the start node which will be included both at the beginning and the end of
/// the path, a node will never be included twice in the path as determined
/// by the `Eq` relationship.
#[allow(dead_code)]
pub fn bfs_loop<N, FN, IN>(start: &N, successors: FN) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    bfs_core(start, successors, |n| n == start, false)
}

/// Visit all nodes that are reachable from a start node. The node will be visited
/// in BFS order, starting from the `start` node and following the order returned
/// by the `successors` function.
///
/// # Examples
///
/// The iterator stops when there are no new nodes to visit:
///
/// ```
/// use pathfinding::prelude::bfs_reach;
///
/// let all_nodes = bfs_reach(3, |_| (1..=5)).collect::<Vec<_>>();
/// assert_eq!(all_nodes, vec![3, 1, 2, 4, 5]);
/// ```
///
/// The iterator can be used as a generator. Here are for examples
/// the multiples of 2 and 3 (although not in natural order but in
/// the order they are discovered by the BFS algorithm):
///
/// ```
/// use pathfinding::prelude::bfs_reach;
///
/// let mut it = bfs_reach(1, |&n| vec![n*2, n*3]).skip(1);
/// assert_eq!(it.next(), Some(2));  // 1*2
/// assert_eq!(it.next(), Some(3));  // 1*3
/// assert_eq!(it.next(), Some(4));  // (1*2)*2
/// assert_eq!(it.next(), Some(6));  // (1*2)*3
/// // (1*3)*2 == 6 which has been seen already
/// assert_eq!(it.next(), Some(9));  // (1*3)*3
/// assert_eq!(it.next(), Some(8));  // ((1*2)*2)*2
/// assert_eq!(it.next(), Some(12)); // ((1*2)*2)*3
/// ```
#[allow(dead_code)]
pub fn bfs_reach<N, FN, IN>(start: N, successors: FN) -> BfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    let (mut to_see, mut seen) = (VecDeque::new(), HashSet::new());
    to_see.push_back(start.clone());
    seen.insert(start);
    BfsReachable {
        to_see,
        seen,
        successors,
    }
}

/// Struct returned by [`bfs_reach`](crate::directed::bfs::bfs_reach).
pub struct BfsReachable<N, FN> {
    to_see: VecDeque<N>,
    seen: HashSet<N>,
    successors: FN,
}

impl<N, FN, IN> Iterator for BfsReachable<N, FN>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(n) = self.to_see.pop_front() {
            for s in (self.successors)(&n) {
                if !self.seen.contains(&s) {
                    self.to_see.push_back(s.clone());
                    self.seen.insert(s);
                }
            }
            Some(n)
        } else {
            None
        }
    }
}
