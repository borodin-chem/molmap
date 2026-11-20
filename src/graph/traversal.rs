// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Implementation of depth-first and breadth-first graph traversal.
//!
//! # Implementation notes
//!
//! Only strong bonds (covalent/dipolar, ionic, metallic) are currently treated
//! as edges.
//!
//! Only atomlikes are currently treated as nodes, even though edges/bonds
//! connect bondables, and those two sets may not be identical. Edges/bonds
//! always _lead_ to nodes/atomlikes, but sometimes via bondables that are not
//! atomlikes.
//!
//! For example, in ferrocene, the central iron atom is bonded to each
//! cyclopentadienyl ring as a whole. Molmap models this by having the π-system
//! be itself bondable, and the bond has that π-system as the bonding partner.
//! However, the π-system is not a node in the graph. The carbon atoms of the Cp
//! ring are the nodes, and each is adjacent to the iron atom, even though they
//! are not directly connected by edges.
//!
//! Currently, an edge is not considered to have a depth, only nodes are. An
//! edge that leads to a node with a depth that exceeds the max allowed depth
//! still gets "explored", but the node is not visited.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    atomic::{AtomMap, Atomic},
    entities::*,
    error::Error,
    traits::{Map, Stores},
};

/// The result of a step in a traversal.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Step {
    /// A previously unexplored edge to a previously unvisited child node has been
    /// explored and the child node visited.
    ///
    /// The visited child node is not a pendant node, and nor has a cycle been closed.
    ///
    /// The newly visited child node may now be the current node i.e. the traversal
    /// may have forward-tracked at the same time. This is indicated by `moved`.
    /// If `moved` is `true` then the traversal is now at a depth of `child_depth`, and
    /// the next step will explore an edge leading out of the child node.
    ///
    /// The parent node may or may not now be fully explored, depending on whether the
    /// explored edge was the final remaining unexplored edge leaving that node; it is
    /// indicated by `parent_fully_explored`.
    VisitChild {
        parent: AnyAtomlike,
        edge: Bond,
        child: AnyAtomlike,
        child_depth: usize,
        parent_fully_explored: bool,
        moved: bool,
    },
    /// A previously unexplored edge to a pendant node has been explored and the pendant
    /// node visited, but the parent node has remained the current node.
    ///
    /// Pendant nodes are nodes with degree one, meaning that there is only one edge
    /// leading in to or out of it: the one between it and the parent node. The pendant
    /// node is thus now fully explored, and will never become the current node.
    ///
    /// A pendant node is most often a hydrogen atom, but may also be e.g. a halogen,
    /// =O, ≡N etc.
    ///
    /// The parent node may or may not now be fully explored, depending on whether the
    /// explored edge was the final remaining unexplored edge leaving that node; it is
    /// indicated by `parent_fully_explored`.
    VisitPendant {
        parent: AnyAtomlike,
        edge: Bond,
        pendant: AnyAtomlike,
        pendant_depth: usize,
        parent_fully_explored: bool,
    },
    /// A previously unexplored edge leading to a **previously visited** node has been
    /// explored, resulting in the closure of a cycle.
    ///
    /// The confluence node (the previously visited node and the destination of the
    /// edge) has not been revisited as part of this. The current node remains the same.
    ///
    /// The depth of the confluence within the graph is calculated in two different ways
    /// and both values are returned: `confluence_original_depth` is the number of edges
    /// that were followed between the root and the confluence originally, while
    /// `confluence_current_depth` is the number of edges that have been followed from
    /// the root to get to this cycle-closing step.
    ///
    /// In a depth-first search, the cycle closing step will always be a loop round to a
    /// node either in the current branch or a distance of 1 from it, so
    /// `confluence_original_depth` will always be lower.
    /// In a breadth-first search, the depth of the confluence is fixed and independent
    /// of the path taken, so both depth values should always be the same.
    ///
    /// Prior to this step, the confluence node will have definitely been on the stack,
    /// as it was only partially explored. That node may or may not now be fully
    /// explored, depending on whether the explored edge was its final remaining
    /// unexplored edge. The explored edge will now have been removed from the queue of
    /// the confluence node on the stack. If the confluence node's queue is now empty,
    /// it will not be visited or explored again, and `confluence_fully_explored` will be
    /// `true` to indicate as such.
    ///
    /// Similarly, the node from which the cycle-closing edge came may or may not now be
    /// fully explored; this is indicated by `current_fully_explored`.
    CloseCycle {
        current: AnyAtomlike,
        edge: Bond,
        confluence: AnyAtomlike,
        /// The depth of the confluence along the original path to it from the root.
        confluence_original_depth: usize,
        /// The depth of the confluence along this now-concluded path to it from the root.
        confluence_current_depth: usize,
        current_fully_explored: bool,
        confluence_fully_explored: bool,
    },
    /// An edge has been explored that led to a child node at a depth that exceeds the
    /// specified maximum depth, so the destination node has been skipped and not visited.
    ///
    /// The current node remains the same (the parent node).
    ///
    /// The parent node may or may not now be fully explored, depending on whether the
    /// explored edge was the final remaining unexplored edge leaving that node; it is
    /// indicated by `parent_fully_explored`.
    SkipTooDeep {
        parent: AnyAtomlike,
        edge: Bond,
        child: AnyAtomlike,
        child_depth: usize,
        parent_fully_explored: bool,
    },
    /// The traversal has moved to a node that was already visited but is only partially
    /// explored, where the entered node has a higher depth than the one left.
    ///
    /// The entered node is not necessarily a child of the left one.
    ///
    /// This means that:
    /// 1. the left node was **fully explored**
    /// 2. the left node will **not be visited again**
    /// 3. both the left and the entered nodes are **not pendant nodes**,
    ///    as these are explored without ever becoming the current node
    ///
    /// The entered node is now the current node, and subsequent steps will explore the
    /// edges leaving the entered node. The entered node is partially explored, in that
    /// the incoming edge will already have been explored, but at least one unexplored
    /// outbound edge remains.
    ///
    /// No nodes or edges have been explored in this step.
    ForwardTrack {
        left: AnyAtomlike,
        entered: AnyAtomlike,
        new_depth: usize,
    },
    /// The traversal has moved to a node that was already visited but is only partially
    /// explored, while remaining at the same depth.
    ///
    /// The entered node does not necessarily have the same parent as the left one.
    ///
    /// This means that:
    /// 1. the left node was **fully explored**
    /// 2. the left node will **not be visited again**
    /// 3. both the left and the entered nodes are **not pendant nodes**,
    ///    as these are explored without ever becoming the current node
    ///
    /// No nodes or edges have been explored in this step.
    SideTrack {
        left: AnyAtomlike,
        entered: AnyAtomlike,
        current_depth: usize,
    },
    /// The traversal has moved to a node that was already visited but is only partially
    /// explored, where the entered node has a lower depth than the one left.
    ///
    /// The entered node is not necessarily a direct ancestor of the left one.
    ///
    /// This means that:
    /// 1. the left node was **fully explored**
    /// 2. the left node will **not be visited again**
    /// 3. both the left and the entered nodes are **not pendant nodes**,
    ///    as these are explored without ever becoming the current node
    ///
    /// No nodes or edges have been explored in this step.
    BackTrack {
        left: AnyAtomlike,
        entered: AnyAtomlike,
        new_depth: usize,
    },
    // This NewRoot variant may be useful in future if we have traversals/iterators that
    // are allowed to jump to other networks after exhausting the initial one
    ///// A new, previously undiscovered node, unconnected to any previously discovered
    ///// node, has been selected, visited, and set as the new current node.
    /////
    ///// The entered node has not been explored at all, and subsequent steps will explore
    ///// the entered node; the exception to this is if the entered node is an isolated
    ///// node (it has degree zero and no edges leading in or out of it), in which case
    ///// the entered node is automatically immediately fully explored.
    //NewRoot {
    //    left: AnyAtomlike,
    //    entered: AnyAtomlike,
    //},
}

#[derive(Clone, Debug)]
struct EdgeQueueItem {
    edge: Bond,
    dest: AnyAtomlike,
    degree: u8,
}

#[derive(Clone, Debug)]
struct NodeQueueItem {
    node: AnyAtomlike,
    depth: usize,
    /// The queue of edges waiting to be explored.
    ///
    /// The edges should be sorted in order of descending priority.
    /// The next edge in the queue -- the one with the next-highest priority --
    /// is then obtained by popping it off the _front_ of the queue.
    edge_queue: VecDeque<EdgeQueueItem>,
}

trait Strategy: Copy {
    type Queue: TraversalQueue;

    /// Whether a visited child node becomes the new current node.
    const MOVE_TO_CHILD_ON_VISIT: bool;

    /// Sorts an edge queue for a node, prioritizing them according to the
    /// strategy's approach.
    ///
    /// The edges should be sorted in _descending_ order of their actual priority,
    /// such that the next edge in the queue -- the one with the next-highest
    /// priority -- is obtained by popping it off the _front_ of the queue.
    ///
    /// The relative order of ties should be maintained.
    fn sort_edges(graph: &AtomMap, node_item: &mut NodeQueueItem);
}

trait TraversalQueue {
    /// Creates a new, empty queue.
    #[allow(unused)]
    fn new() -> Self;

    /// Constructs a new, empty queue with at least the specified capacity.
    #[allow(unused)]
    fn with_capacity(capacity: usize) -> Self;

    /// Returns `true` if there are no nodes in the queue.
    fn is_empty(&self) -> bool;

    /// Returns the number of nodes in the queue.
    fn len(&self) -> usize;

    /// Returns a reference to the node at the front of the queue.
    fn front(&self) -> Option<&NodeQueueItem>;

    /// Returns a mutable reference to the node at the front of the queue.
    fn front_mut(&mut self) -> Option<&mut NodeQueueItem>;

    /// Returns a mutable reference to the specified node's item in the queue, if present.
    fn find_mut(&mut self, node: AnyAtomlike) -> Option<&mut NodeQueueItem>;

    /// Removes the node at the front of the queue.
    fn pop_front(&mut self) -> Option<NodeQueueItem>;

    /// Adds a node to the queue at whatever position is appropriate for the strategy.
    fn push(&mut self, node: NodeQueueItem);
}

/// The stack of nodes waiting to be explored in a depth-first search.
///
/// The next node to be explored is taken from the back, and visited nodes are
/// added by pushing them onto the back (so that they are the next up).
struct DepthFirstStack(Vec<NodeQueueItem>);

impl TraversalQueue for DepthFirstStack {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn front(&self) -> Option<&NodeQueueItem> {
        self.0.last()
    }

    fn front_mut(&mut self) -> Option<&mut NodeQueueItem> {
        self.0.last_mut()
    }

    fn find_mut(&mut self, node: AnyAtomlike) -> Option<&mut NodeQueueItem> {
        self.0.iter_mut().find(|x| x.node == node)
    }

    fn pop_front(&mut self) -> Option<NodeQueueItem> {
        self.0.pop()
    }

    fn push(&mut self, node: NodeQueueItem) {
        self.0.push(node)
    }
}

/// The queue of nodes waiting to be explored in a breadth-first search.
///
/// The next node to be explored is taken from the front, and visited nodes are
/// queued by pushing them onto the back.
struct BreadthFirstQueue(VecDeque<NodeQueueItem>);

impl TraversalQueue for BreadthFirstQueue {
    fn new() -> Self {
        Self(VecDeque::new())
    }

    fn with_capacity(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn front(&self) -> Option<&NodeQueueItem> {
        self.0.front()
    }

    fn front_mut(&mut self) -> Option<&mut NodeQueueItem> {
        self.0.front_mut()
    }

    fn find_mut(&mut self, node: AnyAtomlike) -> Option<&mut NodeQueueItem> {
        self.0.iter_mut().find(|x| x.node == node)
    }

    fn pop_front(&mut self) -> Option<NodeQueueItem> {
        self.0.pop_front()
    }

    fn push(&mut self, node_item: NodeQueueItem) {
        self.0.push_back(node_item)
    }
}

fn default_edge_sort(graph: &AtomMap, node_item: &mut NodeQueueItem) {
    node_item.edge_queue.make_contiguous().sort_by_key(|e| {
        // Lower number for priority means a higher priority
        match e.dest.as_bondable().resolve() {
            ResolvedBondable::Atom(atom) => {
                match graph.data(atom).unwrap().element.atomic_number() {
                    1 => {
                        if e.degree == 1 {
                            1 // Pendant H has highest priority
                        } else {
                            257 // Bridging H has priority according to its Z i.e. third-highest
                        }
                    }
                    6 => 6,              // Carbon has second-highest priority
                    x => x as u16 + 256, // Addition of 256 is a single bit flip 0 -> 1 with no carrying
                }
            }
            ResolvedBondable::Pseudoatom(_pseudoatom) => {
                u16::MAX // Easy to compare as larger
            }
        }
    })
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct DepthFirst;

impl Strategy for DepthFirst {
    type Queue = DepthFirstStack;

    const MOVE_TO_CHILD_ON_VISIT: bool = true;

    fn sort_edges(graph: &AtomMap, node_item: &mut NodeQueueItem) {
        default_edge_sort(graph, node_item);
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct BreadthFirst;

impl Strategy for BreadthFirst {
    type Queue = BreadthFirstQueue;

    const MOVE_TO_CHILD_ON_VISIT: bool = false;

    fn sort_edges(graph: &AtomMap, node_item: &mut NodeQueueItem) {
        default_edge_sort(graph, node_item);
    }
}

/// A traversal of the molecular graph in the form of an iterator over
/// traversal steps.
///
/// The nodes of the graph are all the atoms and pseudoatoms, and the edges are
/// all the strong bonds (covalent/dipolar, ionic, metallic).
///
/// The graph is traversed from the root until all connected nodes with
/// a depth equal to or less than the maximum depth have been visited.
/// As such, if the graph contains multiple disconnected networks, only a subset
/// will be traversed.
///
/// Each iteration returns an instance of [`Step`]. Not every step explores a
/// new edge or node – some just result in a change of the current node.
///
/// The order in which the children of a given node are visited depends on `S`.
/// Using the default [`DepthFirst`] and [`BreadthFirst`] strategies, bonds to
/// pendant (terminal) hydrogen atoms are explored first, followed by carbon
/// atoms, then other atoms by ascending atomic number, then pseudoatoms.
#[derive(Debug)]
#[allow(private_bounds)]
pub struct Traversal<'m, S: Strategy> {
    graph: &'m AtomMap,
    /// All visited nodes, whether partially or fully explored.
    visited_nodes: HashSet<AnyAtomlike>,
    explored_edges: HashSet<Bond>,
    /// The stack of nodes waiting to be explored.
    ///
    /// The next node to be explored is taken from the back, and nodes are added by
    /// pushing them onto the back.
    node_queue: S::Queue,
    max_depth: usize,
}

#[allow(private_bounds)]
impl<'m, S: Strategy> Traversal<'m, S> {
    /// Initializes a traversal of the graph from the indicated atom or pseudoatom.
    ///
    /// The traversal has an implicit maximum depth of `usize::MAX` and is therefore
    /// almost guaranteed to continue until the entire graph has been explored.
    ///
    /// # Errors
    ///
    /// Fails if the graph does not contain any atoms or pseudoatoms at all, or if
    /// the root is not a member of the graph.
    pub fn new<M>(map: &'m M, root: impl Atomlike, _strategy: S) -> Result<Self, Error>
    where
        M: Map + Atomic,
    {
        Self::with_max_depth(map, root, _strategy, usize::MAX)
    }

    /// Initializes a traversal of the graph from the indicated atom or pseudoatom
    /// that stops when the specified depth is exceeded.
    ///
    /// # Errors
    ///
    /// Fails if the graph does not contain any atoms or pseudoatoms at all, or if
    /// the root is not a member of the graph.
    pub fn with_max_depth<M>(
        map: &'m M,
        root: impl Atomlike,
        _strategy: S,
        max_depth: usize,
    ) -> Result<Self, Error>
    where
        M: Map + Atomic,
    {
        let graph = map.as_atomic();
        if !graph.contains(root) {
            return Err(Error::InvalidId(root.as_atomlike().into()));
        }
        let mut traversal = Self {
            graph,
            visited_nodes: HashSet::with_capacity(
                <AtomMap as Stores<Atom>>::count(graph)
                    + <AtomMap as Stores<Pseudoatom>>::count(graph),
            ),
            explored_edges: HashSet::with_capacity(<AtomMap as Stores<Bond>>::count(graph)),
            node_queue: S::Queue::with_capacity(<AtomMap as Stores<Atom>>::count(graph) / 2), // Be generous but assume at least half of nodes/atoms are pendant
            max_depth,
        };
        // Visit root, populate its queue so that we're ready to go
        let root = root.as_atomlike();
        traversal.visited_nodes.insert(root);
        traversal.queue_node(root, 0);
        Ok(traversal)
    }

    /// Returns the node at the front of the queue.
    pub(crate) fn current_node(&self) -> Option<AnyAtomlike> {
        self.node_queue.front().map(|x| x.node)
    }

    /// Returns the depth of the node at the front of the queue.
    pub(crate) fn current_depth(&self) -> Option<usize> {
        self.node_queue.front().map(|x| x.depth)
    }

    /// Returns the current length of the node queue.
    pub(crate) fn queued_nodes(&self) -> usize {
        self.node_queue.len()
    }

    /// Creates an edge queue for a node, populates it by discovering outgoing edges,
    /// then adds the node to the node queue.
    fn queue_node(&mut self, node: AnyAtomlike, depth: usize) {
        let mut queue: VecDeque<EdgeQueueItem> = VecDeque::with_capacity(6);
        let outward_bonds = match node.resolve() {
            ResolvedAtomlike::Atom(atom) => &self.graph.data(atom).unwrap().bonds,
            ResolvedAtomlike::Pseudoatom(pseudoatom) => &self.graph.data(pseudoatom).unwrap().bonds,
        };
        for &bond in outward_bonds {
            // Skip bond if it has already been explored (i.e. we came in to the
            // node that way)
            if self.explored_edges.contains(&bond) {
                continue;
            }
            let bond_data = self.graph.data(bond).unwrap();
            // Skip bond if it is weak, weak bonds don't count as edges
            if !bond_data.bond_type.is_strong() {
                continue;
            }
            // OK, this bond is a valid candidate for exploration, so we add it to the queue.
            // We will sort the queue after fully assembling it.
            // First, need to work out what the other end of this bond is.
            let dest_bondable = if bond_data.start == node.as_bondable() {
                bond_data.end
            } else if bond_data.end == node.as_bondable() {
                bond_data.start
            } else {
                unreachable!("Currently, the node will always be one of the two bonding partners")
            };
            let (dest, degree) = match dest_bondable.resolve() {
                ResolvedBondable::Atom(atom) => {
                    let degree = self
                        .graph
                        .data(atom)
                        .unwrap()
                        .bonds
                        .iter()
                        .filter(|&&b| self.graph.data(b).unwrap().bond_type.is_strong())
                        .count();
                    (atom.as_atomlike(), degree)
                }
                ResolvedBondable::Pseudoatom(pseudoatom) => {
                    let degree = self
                        .graph
                        .data(pseudoatom)
                        .unwrap()
                        .bonds
                        .iter()
                        .filter(|&&b| self.graph.data(b).unwrap().bond_type.is_strong())
                        .count();
                    (pseudoatom.as_atomlike(), degree)
                }
            };
            queue.push_back(EdgeQueueItem {
                edge: bond,
                dest,
                degree: degree.try_into().expect(
                    "Seems reasonable to assume that no node will have a degree of more than 255",
                ),
            })
        }
        let mut node_item = NodeQueueItem {
            node,
            depth,
            edge_queue: queue,
        };
        S::sort_edges(self.graph, &mut node_item);
        self.node_queue.push(node_item);
    }

    /// Explores the next edge in the current node's edge queue and the destination node
    /// in the knowledge that the node is a child node, visiting it and adding an edge
    /// queue for it to the node queue.
    ///
    /// The current node may remain the same, or the traversal may move to the child
    /// node, depending on the value of `S::MOVE_TO_CHILD_ON_VISIT`.
    ///
    /// However, if the node is a pendant node, the current node will always remain the
    /// same; the pendant node will then be fully explored and will never become the
    /// current node.
    ///
    /// This method relies on the destination node not having been previously visited.
    ///
    /// # Panics
    ///
    /// Panics if either the node queue or the current node's edge queue are empty.
    fn visit_child(&mut self) -> Step {
        let parent = self.current_node().unwrap();
        let parent_queue = &mut self.node_queue.front_mut().unwrap().edge_queue;
        let explored_edge = parent_queue.pop_front().unwrap();
        let parent_fully_explored = parent_queue.is_empty();
        let edge = explored_edge.edge;
        let child = explored_edge.dest;
        let child_depth = self.current_depth().unwrap() + 1;
        self.explored_edges.insert(edge);
        // We have visited (and maybe moved to) a completely unexplored, unvisited node
        self.visited_nodes.insert(child);
        // Node is a pendant if there is only one edge in
        // IMPORTANT NOTE TO MAINTAINERS: this will no longer hold when bonding
        // bonding systems are introduced, as it won't be guaranteed that one bond
        // out means only one edge in. It will be necessary to check if the pendant
        // is a member of a bonding system.
        if explored_edge.degree == 1 {
            Step::VisitPendant {
                parent,
                edge,
                pendant: child,
                pendant_depth: child_depth,
                parent_fully_explored,
            }
        } else {
            self.queue_node(child, child_depth);
            Step::VisitChild {
                parent,
                edge,
                child,
                child_depth,
                parent_fully_explored,
                moved: S::MOVE_TO_CHILD_ON_VISIT,
            }
        }
    }

    /// Explores the next edge in the queue in the knowledge that the destination
    /// node is a previously visited "confluence" node, removing the explored edge
    /// from the queue of the confluence node on the stack, while keeping the
    /// current node the same.
    ///
    /// # Panics
    ///
    /// Panics if either the stack or queue are empty, or if the confluence node
    /// does not have a queue somewhere back along the stack.
    fn close_cycle(&mut self) -> Step {
        let current = self.current_node().unwrap();
        let current_queue = &mut self.node_queue.front_mut().unwrap().edge_queue;
        let explored_edge = current_queue.pop_front().unwrap();
        let current_fully_explored = current_queue.is_empty();
        let edge = explored_edge.edge;
        let confluence = explored_edge.dest;
        let confluence_current_depth = self.current_depth().unwrap() + 1;
        self.explored_edges.insert(edge);
        // We have just explored one of the edges that the confluence had in its queue,
        // so we should find its item in the stack and remove this edge from its queue
        let confluence_on_stack = self.node_queue.find_mut(confluence).expect(
            "Must be on stack, as the edge we followed to get to it was unexplored until now",
        );
        if let Some(edge_index) = confluence_on_stack
            .edge_queue
            .iter()
            .position(|x| x.edge == edge)
        {
            confluence_on_stack.edge_queue.remove(edge_index);
        }
        let confluence_original_depth = confluence_on_stack.depth;
        let confluence_fully_explored = confluence_on_stack.edge_queue.is_empty();
        Step::CloseCycle {
            current,
            edge,
            confluence,
            confluence_original_depth,
            confluence_current_depth,
            current_fully_explored,
            confluence_fully_explored,
        }
    }

    /// Explores the next edge in the queue in the knowledge that the depth of the
    /// destination node exceeds the maximum depth configured for the search.
    ///
    /// The destination node is not visited or explored at all, but the edge is
    /// considered explored. The current node (the parent node) is kept the same.
    ///
    /// # Panics
    ///
    /// Panics if either the node queue or the current node's edge queue are empty.
    fn skip_child(&mut self) -> Step {
        let parent = self.current_node().unwrap();
        let parent_queue = &mut self.node_queue.front_mut().unwrap().edge_queue;
        let explored_edge = parent_queue.pop_front().unwrap();
        let parent_fully_explored = parent_queue.is_empty();
        let edge = explored_edge.edge;
        let child = explored_edge.dest;
        let child_depth = self.current_depth().unwrap() + 1;
        self.explored_edges.insert(edge);
        // Child node doesn't get visited and never will be, and it never gets put
        // into the node queue even if it isn't a pendant node
        Step::SkipTooDeep {
            parent,
            edge,
            child,
            child_depth,
            parent_fully_explored,
        }
    }

    /// Tracks along the node queue to the next unexhausted node, or returns `None`
    /// if none remain and the node queue is now empty.
    ///
    /// One of `Step::SideTrack`, `Step::BackTrack`, or `Step::ForwardTrack` are
    /// returned depending on whether the node arrived at is at the same depth, a
    /// higher depth, or a lower depth than the departed node.
    ///
    /// Any encountered node queue items with no remaining unexplored edges are removed
    /// from the node queue.
    fn track(&mut self) -> Option<Step> {
        let left = self.current_node().unwrap();
        let left_depth = self.current_depth().unwrap();
        while let Some(front) = self.node_queue.front() {
            if front.edge_queue.is_empty() {
                self.node_queue.pop_front();
            } else {
                // Found a queued node that is only partially explored
                let entered = front.node;
                let new_depth = front.depth;
                let step = match new_depth.cmp(&left_depth) {
                    std::cmp::Ordering::Less => Step::BackTrack {
                        left,
                        entered,
                        new_depth,
                    },
                    std::cmp::Ordering::Equal => Step::SideTrack {
                        left,
                        entered,
                        current_depth: new_depth,
                    },
                    std::cmp::Ordering::Greater => {
                        // Should only ever go one step deeper
                        debug_assert_eq!(new_depth, left_depth + 1);
                        Step::ForwardTrack {
                            left,
                            entered,
                            new_depth,
                        }
                    }
                };
                return Some(step);
            }
        }
        // No unexhausted nodes encountered
        None
    }
}

impl<'m, S: Strategy> Iterator for Traversal<'m, S> {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        // Possible variants of Step and the scenarios in which they will be returned:
        // - Child        - next edge to explore leads to unvisited node with degree > 1
        // - Pendant      - next edge to explore leads to unvisited node with degree == 1
        // - CycleClosure - next edge to explore leads to already visited node, a cycle is closed
        // - SkipTooDeep  - next edge to explore leads to an unvisited node that is deeper than max_depth
        // - *Track       - no remaining unexplored edges from node, but stack isn't empty
        //
        // If the stack is empty, or a backtrack fails to find any nodes with edges left to
        // explore, traversal is finished.

        if self.node_queue.is_empty() {
            None
        } else {
            let current_queue = &self.node_queue.front().unwrap().edge_queue;
            if current_queue.is_empty() {
                // Nothing in the queue, so the current node is fully explored -
                // back(/side/forward)track and see if we can find one that isn't yet
                self.track()
            } else {
                // The current node has at least one edge to explore, and each of
                // the helper methods we call will explore the next in the queue
                let next = current_queue.front().unwrap();
                // The edge should not have been previously explored
                debug_assert!(!self.explored_edges.contains(&next.edge));
                // We are closing a cycle if the node led to by the edge is one that
                // has already been visited
                let step = if self.visited_nodes.contains(&next.dest) {
                    self.close_cycle()
                } else {
                    // The destination node is a child (pendants are also children),
                    // which means the depth will be higher -- check that it doesn't
                    // exceed the max depth, because if so, we shouldn't visit it
                    if self
                        .current_depth()
                        .expect("We know that the queue isn't empty")
                        == self.max_depth
                    {
                        self.skip_child()
                    } else {
                        self.visit_child()
                    }
                };
                Some(step)
            }
        }
    }
}

/// The result of a step in a bidirectional traversal.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum BidirectionalStep {
    /// A node has been visited after tracing a path to it from the initial node.
    VisitForwards {
        parent: AnyAtomlike,
        child: AnyAtomlike,
        child_depth: usize,
    },
    /// A node has been visited after tracing a path to it from the goal node.
    VisitBackwards {
        parent: AnyAtomlike,
        child: AnyAtomlike,
        child_depth: usize,
    },
    /// A node has been identified that can be reached from both starting nodes.
    MeetAtCommon {
        parent: AnyAtomlike,
        common: AnyAtomlike,
        distance: usize,
    },
}

/// A traversal of the molecular graph from two root nodes simultaneously, usually
/// with the goal of finding a path, or the shortest path, between them.
///
/// Each iteration returns a [`BidirectionalStep`]. The distance to the visited node
/// is always returned along with its parent, so that a caller can at the end deduce
/// the full path taken along the shortest path.
///
/// Iteration ends once at least one of the traversals is exhausted; unless the
/// search was deliberately continued after a common node was identified, this
/// indicates that there is no path connecting the initial and goal nodes.
///
/// The traversal that is advanced at each step is always the one with the fewest
/// nodes in the queue, which helps to reduce the time required for negative
/// results.
///
/// See [`Traversal`] for more details on how each individual traversal works.
#[allow(private_bounds)]
pub struct BidirectionalSearch<'m> {
    initial: AnyAtomlike,
    goal: AnyAtomlike,
    fwds: Traversal<'m, BreadthFirst>,
    bwds: Traversal<'m, BreadthFirst>,
    // Keep track of the distance to each visited node
    visited_fwds: HashMap<AnyAtomlike, usize>,
    visited_bwds: HashMap<AnyAtomlike, usize>,
}

#[allow(private_bounds)]
impl<'m> BidirectionalSearch<'m> {
    /// Initializes a bidirectional search between the indicated atoms or pseudoatoms.
    ///
    /// The traversals have an implicit maximum depth of `usize::MAX` and are therefore
    /// almost guaranteed to find a connecting path, if there is one.
    ///
    /// # Errors
    ///
    /// Fails if `initial == goal`.
    ///
    /// Fails if the graph does not contain any atoms or pseudoatoms at all, or if
    /// either root is not a member of the graph.
    pub fn new<M>(map: &'m M, initial: impl Atomlike, goal: impl Atomlike) -> Result<Self, Error>
    where
        M: Map + Atomic,
    {
        let initial = initial.as_atomlike();
        let goal = goal.as_atomlike();
        if initial == goal {
            Err(Error::Disallowed(String::from(
                "Cannot conduct a bidirectional search where the initial node and goal node are the same node!",
            )))
        } else {
            let mut search = Self {
                initial,
                goal,
                fwds: Traversal::new(map, initial, BreadthFirst)?,
                bwds: Traversal::new(map, goal, BreadthFirst)?,
                visited_fwds: HashMap::new(),
                visited_bwds: HashMap::new(),
            };
            // Put the roots themselves to be in the records so that they come up if queried
            search.visited_fwds.insert(initial, 0);
            search.visited_bwds.insert(goal, 0);
            Ok(search)
        }
    }
}

impl<'m> Iterator for BidirectionalSearch<'m> {
    type Item = BidirectionalStep;

    fn next(&mut self) -> Option<Self::Item> {
        // Note to future readers
        // The traversals themselves are currently just basic breadth-first searches as
        // implemented above. In particular, that means that children are prioritized in
        // the order: pendant H > C > other H > other elements (in element order).
        // One way that this shortest-path algorithm might be made more efficient would
        // be to instead prioritize children based on the index part of the ID - as in,
        // pursue a path that brings the index of the current node closer to that of the
        // goal node.
        // For maps where the indices are entirely random this would have no benefit,
        // but in reality of course they have usually been assigned or handed out at
        // least semi-systematically.
        // It should only affect the priority within each depth level, though.

        // Keep iterating until we get to an interesting step
        loop {
            let (next_is_fwds, next) = {
                // We iterate the traversal with the fewest nodes queued to try and
                // keep the trees from expanding any more than necessary
                let first_try_fwds = self.fwds.queued_nodes() < self.bwds.queued_nodes();
                let first_try = if first_try_fwds {
                    self.fwds.next()
                } else {
                    self.bwds.next()
                };
                if let Some(n) = first_try {
                    (first_try_fwds, n)
                } else {
                    // If either queue is empty, the nodes are clearly not connected
                    // (Or, the caller continued the search going until the network was fully explored)
                    break None;
                }
            };
            match next {
                Step::VisitChild {
                    parent,
                    child,
                    child_depth,
                    ..
                } => {
                    let depth_in_other_direction = if next_is_fwds {
                        self.visited_fwds.insert(child, child_depth);
                        self.visited_bwds.get(&child)
                    } else {
                        self.visited_bwds.insert(child, child_depth);
                        self.visited_fwds.get(&child)
                    };
                    if let Some(other_depth) = depth_in_other_direction {
                        // Found a common node!
                        let total_distance = child_depth + other_depth;
                        break Some(BidirectionalStep::MeetAtCommon {
                            parent,
                            common: child,
                            distance: total_distance,
                        });
                    } else {
                        if next_is_fwds {
                            break Some(BidirectionalStep::VisitForwards {
                                parent,
                                child,
                                child_depth,
                            });
                        } else {
                            break Some(BidirectionalStep::VisitBackwards {
                                parent,
                                child,
                                child_depth,
                            });
                        }
                    }
                }
                Step::VisitPendant {
                    parent,
                    pendant,
                    pendant_depth,
                    ..
                } => {
                    // Don't need to record it, as the only way in is from a parent
                    // that would then also be common to both trees
                    // Only way the pendant is relevant is if it happens to be the
                    // other root
                    if pendant == self.initial || pendant == self.goal {
                        break Some(BidirectionalStep::MeetAtCommon {
                            parent,
                            common: pendant,
                            distance: pendant_depth,
                        });
                    }
                }
                Step::CloseCycle { .. } => continue, // Not interesting - confluence visited before and this path is longer
                Step::SideTrack { .. } | Step::ForwardTrack { .. } | Step::BackTrack { .. } => {
                    continue;
                }
                Step::SkipTooDeep { .. } => break None, // Shouldn't happen ever anyway, but just in case!
            }
        }
    }
}

/// Calculates the shortest distance from one node to the other using a
/// bidirectional breadth-first search.
///
/// Returns `None` if `a` and `b` are not connected at all.
///
/// Also returns `None` if the search initialization fails (see
/// [`BidirectionalSearch::new`]).
///
/// The traversal that is advanced at each step is always the one with the fewest
/// nodes in the queue, which helps to reduce the time required for negative
/// results.
#[allow(unused)]
pub fn shortest_distance<M>(map: &M, initial: impl Atomlike, goal: impl Atomlike) -> Option<usize>
where
    M: Map + Atomic,
{
    let initial = initial.as_atomlike();
    let goal = goal.as_atomlike();
    if initial == goal {
        return Some(0);
    }
    let mut search = BidirectionalSearch::new(map, initial, goal).ok()?;
    let mut shortest = None;
    for step in search {
        match step {
            BidirectionalStep::MeetAtCommon {
                parent,
                common,
                distance,
            } => {
                if let Some(record) = shortest {
                    if distance < record {
                        // New shortest path found
                        shortest = Some(distance);
                    } else {
                        // If this path is considerably longer than the record then it's
                        // probably safe to assume that the record will stand forever,
                        // no matter how long we search for, so we can halt the search.
                        // That does of course leave us to ask what should be considered
                        // as "considerably longer"...
                        if distance - record > 2 {
                            return Some(record);
                        }
                    }
                } else {
                    shortest = Some(distance);
                    continue;
                }
            }
            _ => continue,
        }
    }
    shortest
}

/// Determines if two nodes are connected using a bidirectional breadth-first search.
///
/// Similar to [`shortest_distance`], but returns `true` as soon as a path is found
/// without bothering to ensure that it is the shortest path.
///
/// # Errors
///
/// Fails if the search initialization fails (see [`BidirectionalSearch::new`]).
#[allow(unused)]
pub fn are_connected<M>(map: &M, initial: impl Atomlike, goal: impl Atomlike) -> Result<bool, Error>
where
    M: Map + Atomic,
{
    let initial = initial.as_atomlike();
    let goal = goal.as_atomlike();
    if initial == goal {
        return Ok(true);
    }
    let mut search = BidirectionalSearch::new(map, initial, goal)?;
    for step in search {
        match step {
            BidirectionalStep::MeetAtCommon { .. } => return Ok(true),
            _ => continue,
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use crate::{bonding::BondType, elements::Element};

    use super::*;

    const ETHANE: &str = include_str!("../../tests/fixtures/atomic/ethane.ron");
    const PROPANE: &str = include_str!("../../tests/fixtures/atomic/propane.ron");
    const CYCLOHEXANE: &str = include_str!("../../tests/fixtures/atomic/cyclohexane.ron");

    #[test]
    fn invalid_root() -> Result<(), Error> {
        let mut m: AtomMap = ron::from_str(ETHANE).unwrap();
        let h1: Atom = "a2.1".parse()?;
        m.delete_atom(h1);
        let new_h = m.add_atom(Element::H);
        assert_ne!(new_h, h1);
        assert_eq!(new_h, "a2.3".parse()?);
        assert!(Traversal::new(&m, h1, DepthFirst).is_err());
        Ok(())
    }

    /// Tests that specifically verify the behaviour of depth-first traversal
    mod depth_first {
        use std::str::FromStr;

        use super::*;

        #[test]
        fn simple_ethane() -> Result<(), Error> {
            // Traverse ethane
            let m: AtomMap = ron::from_str(ETHANE).unwrap();
            // All atoms in file are serialized as version 1
            // a1 is C, a2-4 are its H, a5 is the other C, a6-8 are its H
            // C–C bond is b7.1
            // Start at one of the hydrogen atoms (but the second H in the file,
            // to make it more interesting)
            let mut traversal =
                Traversal::new(&m, AnyAtomlike::from_str("a3")?, DepthFirst).unwrap();
            assert_eq!(
                traversal.next(),
                Some(Step::VisitChild {
                    parent: "a3".parse()?,
                    edge: "b2".parse()?,
                    child: "a1".parse()?,
                    child_depth: 1,
                    parent_fully_explored: true,
                    moved: true,
                })
            );
            // Pendant Hs should be explored first, in insertion order
            assert_eq!(
                traversal.next(),
                Some(Step::VisitPendant {
                    parent: "a1".parse()?,
                    edge: "b1".parse()?,
                    pendant: "a2".parse()?,
                    pendant_depth: 2,
                    parent_fully_explored: false
                })
            );
            assert_eq!(
                traversal.next(),
                Some(Step::VisitPendant {
                    parent: "a1".parse()?,
                    edge: "b3".parse()?,
                    pendant: "a4".parse()?,
                    pendant_depth: 2,
                    parent_fully_explored: false
                })
            );
            // Then we move to the next carbon in the chain
            assert_eq!(
                traversal.next(),
                Some(Step::VisitChild {
                    parent: "a1".parse()?,
                    edge: "b7".parse()?,
                    child: "a5".parse()?,
                    child_depth: 2,
                    parent_fully_explored: true,
                    moved: true
                })
            );
            // Then the other H atoms, again in insertion order
            assert_eq!(
                traversal.next(),
                Some(Step::VisitPendant {
                    parent: "a5".parse()?,
                    edge: "b4".parse()?,
                    pendant: "a6".parse()?,
                    pendant_depth: 3,
                    parent_fully_explored: false
                })
            );
            assert_eq!(
                traversal.next(),
                Some(Step::VisitPendant {
                    parent: "a5".parse()?,
                    edge: "b5".parse()?,
                    pendant: "a7".parse()?,
                    pendant_depth: 3,
                    parent_fully_explored: false
                })
            );
            assert_eq!(
                traversal.next(),
                Some(Step::VisitPendant {
                    parent: "a5".parse()?,
                    edge: "b6".parse()?,
                    pendant: "a8".parse()?,
                    pendant_depth: 3,
                    parent_fully_explored: true
                })
            );
            // Nothing left to traverse
            assert!(traversal.next().is_none());
            Ok(())
        }

        #[test]
        fn simple_propane() -> Result<(), Error> {
            // Traverse propane, starting at the carbon in the middle of the chain
            let m: AtomMap = ron::from_str(PROPANE).unwrap();
            let c1: AnyAtomlike = "a1".parse()?;
            let c2: AnyAtomlike = "a5".parse()?;
            let c3: AnyAtomlike = "a8".parse()?;
            let traversal = Traversal::new(&m, c2, DepthFirst).unwrap();
            let mut explored_edges: Vec<Bond> = Vec::new();
            let mut visited_nodes: Vec<AnyAtomlike> = vec![c2];
            let mut backtracks: Vec<usize> = Vec::new();
            let mut cycle_closures: Vec<usize> = Vec::new();
            for (i, step) in traversal.enumerate() {
                match step {
                    Step::VisitChild { edge, child, .. } => {
                        explored_edges.push(edge);
                        visited_nodes.push(child);
                    }
                    Step::VisitPendant { edge, pendant, .. } => {
                        explored_edges.push(edge);
                        visited_nodes.push(pendant);
                    }
                    Step::CloseCycle { edge, .. } => {
                        explored_edges.push(edge);
                        cycle_closures.push(i);
                    }
                    Step::BackTrack { .. } => backtracks.push(i),
                    _ => unreachable!(),
                }
            }
            assert_eq!(explored_edges.len(), 10);
            assert_eq!(visited_nodes.len(), 11);
            // Should visit the two pendant Hs on C2 first, then move forwards to C1,
            // then explore C1 and its three Hs, then backtrack to C2 => 7th step is the backtrack
            assert_eq!(backtracks, [6]);
            assert!(cycle_closures.is_empty());
            // Confirm that we did indeed visit C1 before C3
            assert!(
                visited_nodes.iter().position(|&x| x == c1).unwrap()
                    < visited_nodes.iter().position(|&x| x == c3).unwrap()
            );
            Ok(())
        }

        #[test]
        fn cyclic() -> Result<(), Error> {
            // Traverse cyclohexane
            let m: AtomMap = ron::from_str(CYCLOHEXANE).unwrap();
            let carbons: Vec<AnyAtomlike> = m
                .iter::<Atom>()
                .filter(|&a| m.view(a).unwrap().element() == Element::C)
                .map(|a| a.as_atomlike())
                .collect();
            let traversal = Traversal::new(&m, carbons[0], DepthFirst).unwrap();
            let mut explored_edges: Vec<Bond> = Vec::new();
            let mut visited_nodes: Vec<AnyAtomlike> = vec![carbons[0]];
            let mut backtracks: Vec<usize> = Vec::new();
            let mut cycle_closures: Vec<usize> = Vec::new();
            for (i, step) in traversal.enumerate() {
                match step {
                    Step::VisitChild { edge, child, .. } => {
                        explored_edges.push(edge);
                        visited_nodes.push(child);
                    }
                    Step::VisitPendant { edge, pendant, .. } => {
                        explored_edges.push(edge);
                        visited_nodes.push(pendant);
                    }
                    Step::CloseCycle { edge, .. } => {
                        explored_edges.push(edge);
                        cycle_closures.push(i);
                    }
                    Step::BackTrack { .. } => backtracks.push(i),
                    _ => unreachable!(),
                }
            }
            assert_eq!(explored_edges.len(), 18);
            assert_eq!(visited_nodes.len(), 18);
            // As search is depth-first and pendant H is explored before the carbon chain,
            // the ring should only have been closed in the very last step
            // No backtracking occurs so total steps = 5 forward + (6 * 2 pendant) + 1 cycle-closing
            assert_eq!(cycle_closures, [17]);
            Ok(())
        }

        #[test]
        fn unconnected_h2o() {
            // Ensure that only the connected network is traversed
            let mut m = AtomMap::new();
            // Two water molecules
            let (o1, _bonds) = m.add_atom_and_saturate(Element::O);
            let (o2, _bonds) = m.add_atom_and_saturate(Element::O);
            let mut traversal = Traversal::new(&m, o1, DepthFirst).unwrap();
            for _ in &mut traversal {}
            // Check that O of second water molecule was never visited
            assert!(!traversal.visited_nodes.contains(&o2.as_atomlike()));
        }

        #[test]
        fn h_bond_doesnt_connect_h2o() {
            let mut m = AtomMap::new();
            // Two water molecules
            let (o1, o1_h_bonds) = m.add_atom_and_saturate(Element::O);
            let o1h1 = o1_h_bonds.first().unwrap().to_owned();
            let (o2, _bonds) = m.add_atom_and_saturate(Element::O);
            let h1 = m
                .view(o1h1)
                .unwrap()
                .partners()
                .into_iter()
                .find(|&x| x != o1.as_bondable()) // Find the bonding partner that isn't O1
                .unwrap();
            // Connect via a hydrogen bond
            m.add_bond_unchecked(BondType::Hydrogen, o2, h1);
            let mut traversal = Traversal::new(&m, o1, DepthFirst).unwrap();
            for _ in &mut traversal {}
            // Check that O of second water molecule was never visited
            assert!(!traversal.visited_nodes.contains(&o2.as_atomlike()));
        }
    }

    mod breadth_first {
        use super::*;

        #[test]
        fn simple_propane() -> Result<(), Error> {
            // Traverse propane, starting at the carbon in the middle of the chain
            let m: AtomMap = ron::from_str(PROPANE).unwrap();
            let c1: AnyAtomlike = "a1".parse()?;
            let h1: AnyAtomlike = "a2".parse()?;
            let h2: AnyAtomlike = "a3".parse()?;
            let h3: AnyAtomlike = "a4".parse()?;
            let c2: AnyAtomlike = "a5".parse()?;
            let h4: AnyAtomlike = "a6".parse()?;
            let h5: AnyAtomlike = "a7".parse()?;
            let c3: AnyAtomlike = "a8".parse()?;
            let h6: AnyAtomlike = "a9".parse()?;
            let h7: AnyAtomlike = "a10".parse()?;
            let h8: AnyAtomlike = "a11".parse()?;
            let traversal = Traversal::new(&m, c2, BreadthFirst).unwrap();
            let mut explored_edges: Vec<Bond> = Vec::new();
            let mut visited_nodes: Vec<AnyAtomlike> = vec![c2.as_atomlike()];
            let mut cycle_closures: Vec<usize> = Vec::new();
            for (i, step) in traversal.enumerate() {
                match step {
                    Step::VisitChild { edge, child, .. } => {
                        explored_edges.push(edge);
                        visited_nodes.push(child);
                    }
                    Step::VisitPendant { edge, pendant, .. } => {
                        explored_edges.push(edge);
                        visited_nodes.push(pendant);
                    }
                    Step::CloseCycle { edge, .. } => {
                        explored_edges.push(edge);
                        cycle_closures.push(i);
                    }
                    Step::ForwardTrack { .. } | Step::SideTrack { .. } => continue,
                    _ => unreachable!(),
                }
            }
            assert_eq!(explored_edges.len(), 10);
            assert_eq!(visited_nodes.len(), 11);
            assert!(cycle_closures.is_empty());
            // Should visit the two pendant Hs on C2 first, then C1, then C3,
            // then move to C1 and visit its three Hs, then move to C3 and its three Hs
            assert_eq!(
                visited_nodes,
                [c2, h4, h5, c1, c3, h1, h2, h3, h6, h7, h8].map(Into::into)
            );
            Ok(())
        }

        #[test]
        fn cyclic() {
            // Traverse cyclohexane
            let m: AtomMap = ron::from_str(CYCLOHEXANE).unwrap();
            let carbons: Vec<AnyAtomlike> = m
                .iter::<Atom>()
                .filter(|&a| m.view(a).unwrap().element() == Element::C)
                .map(|a| a.as_atomlike())
                .collect();
            let traversal = Traversal::new(&m, carbons[0], BreadthFirst).unwrap();
            let mut explored_edges: Vec<Bond> = Vec::new();
            let mut visited_nodes: Vec<AnyAtomlike> = vec![carbons[0]];
            let mut depths: Vec<usize> = vec![0];
            let mut cycle_closures: Vec<usize> = Vec::new();
            let mut cycle_closure_depth: usize = 0;
            let mut step_count: usize = 0;
            for (i, step) in traversal.enumerate() {
                step_count += 1;
                match step {
                    Step::VisitChild {
                        edge,
                        child,
                        child_depth,
                        ..
                    } => {
                        explored_edges.push(edge);
                        visited_nodes.push(child);
                        depths.push(child_depth);
                    }
                    Step::VisitPendant {
                        edge,
                        pendant,
                        pendant_depth,
                        ..
                    } => {
                        explored_edges.push(edge);
                        visited_nodes.push(pendant);
                        depths.push(pendant_depth);
                    }
                    Step::CloseCycle {
                        edge,
                        confluence_current_depth,
                        ..
                    } => {
                        explored_edges.push(edge);
                        cycle_closures.push(i);
                        cycle_closure_depth = confluence_current_depth;
                    }
                    Step::ForwardTrack { .. } | Step::SideTrack { .. } => continue,
                    _ => unreachable!(),
                }
            }
            assert_eq!(explored_edges.len(), 18);
            assert_eq!(visited_nodes.len(), 18);
            assert_eq!(visited_nodes[0], carbons[0]);
            // then 2 Hs on C0, then C1, then C5 (all at depth 1)
            assert_eq!(visited_nodes[3], carbons[1]);
            assert_eq!(visited_nodes[4], carbons[5]);
            // then 2 Hs on C1, then C2 (all at depth 2)
            assert_eq!(visited_nodes[7], carbons[2]);
            // then 2 Hs on C5, then C4 (all at depth 2)
            assert_eq!(visited_nodes[10], carbons[4]);
            // then 2 Hs on C2, then C3 (all at depth 3)
            assert_eq!(visited_nodes[13], carbons[3]);
            // then 2 Hs on C4 (at depth 3), then the ring closure is also depth 3
            // including the moves from C0 -> C1 -> C5 -> C2 -> C4, there have been
            // 15 visiting steps + 4 moves = 19 steps, so the ring closure is 20th step
            assert_eq!(cycle_closures, [19]);
            // then, after final move to C3, 2 Hs on C3 (at depth 4)
            // => 23 total steps
            assert_eq!(step_count, 23);
            assert_eq!(
                depths,
                [0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4],
            );
            assert_eq!(cycle_closure_depth, 3);
        }

        #[test]
        fn unconnected_h2o() {
            // Ensure that only the connected network is traversed
            let mut m = AtomMap::new();
            // Two water molecules
            let o1 = m.add_atom(Element::O);
            let o2 = m.add_atom(Element::O);
            for o in [o1, o2] {
                for _ in 0..2 {
                    let h = m.add_atom(Element::H);
                    m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, o, h);
                }
            }
            let mut traversal = Traversal::new(&m, o1, BreadthFirst).unwrap();
            for _ in &mut traversal {}
            // Check that O of second water molecule was never visited
            assert!(!traversal.visited_nodes.contains(&o2.as_atomlike()));
        }

        #[test]
        fn h_bond_doesnt_connect_h2o() {
            let mut m = AtomMap::new();
            // Two water molecules
            let (o1, o1_h_bonds) = m.add_atom_and_saturate(Element::O);
            let o1h1 = o1_h_bonds.first().unwrap().to_owned();
            let (o2, _bonds) = m.add_atom_and_saturate(Element::O);
            let h1 = m
                .view(o1h1)
                .unwrap()
                .partners()
                .into_iter()
                .find(|&x| x != o1.as_bondable()) // Find the bonding partner that isn't O1
                .unwrap();
            // Connect via a hydrogen bond
            m.add_bond_unchecked(BondType::Hydrogen, o2, h1);
            let mut traversal = Traversal::new(&m, o1, BreadthFirst).unwrap();
            for _ in &mut traversal {}
            // Check that O of second water molecule was never visited
            assert!(!traversal.visited_nodes.contains(&o2.as_atomlike()));
        }
    }

    mod shortest_path {
        use super::*;

        #[test]
        fn simple_propane() -> Result<(), Error> {
            let m: AtomMap = ron::from_str(PROPANE).unwrap();
            let h1: AnyAtomlike = "a2".parse()?;
            let c1: AnyAtomlike = "a1".parse()?;
            let c2: AnyAtomlike = "a5".parse()?;
            let c3: AnyAtomlike = "a8".parse()?;
            let h10: AnyAtomlike = "a11".parse()?;
            // Distance between an atom and itself is zero
            assert_eq!(shortest_distance(&m, c1, c1).unwrap(), 0);
            // Distance between the two terminal carbons
            assert_eq!(shortest_distance(&m, c1, c3).unwrap(), 2);
            // Between central and terminal
            assert_eq!(shortest_distance(&m, c1, c2).unwrap(), 1);
            // Between terminal C and one of its Hs
            assert_eq!(shortest_distance(&m, c1, h1).unwrap(), 1);
            // Between central C and one of the terminal C's Hs
            assert_eq!(shortest_distance(&m, c2, h1).unwrap(), 2);
            // Geminal Hs
            assert_eq!(shortest_distance(&m, h1, "a3".parse::<Atom>()?).unwrap(), 2);
            // Vicinal Hs
            assert_eq!(
                shortest_distance(&m, h10, "a6".parse::<Atom>()?).unwrap(),
                3
            );
            // Between hydrogens at either end
            assert_eq!(shortest_distance(&m, h1, h10).unwrap(), 4);
            Ok(())
        }

        #[test]
        fn disconnected_propane() -> Result<(), Error> {
            let mut m: AtomMap = ron::from_str(PROPANE).unwrap();
            let h1: AnyAtomlike = "a2".parse()?;
            let c1: AnyAtomlike = "a1".parse()?;
            let c2: AnyAtomlike = "a5".parse()?;
            let c3: AnyAtomlike = "a8".parse()?;
            let h10: AnyAtomlike = "a11".parse()?;
            // Delete the C1-C2 bond
            m.delete_bond("b9".parse()?);
            // Now repeat the tests above; should get different results
            // Distance between an atom and itself is still zero
            assert_eq!(shortest_distance(&m, c1, c1).unwrap(), 0);
            // C1 and C2 are of course now disconnected
            assert!(shortest_distance(&m, c1, c2).is_none());
            // The two terminal carbons are also now disconnected
            assert!(shortest_distance(&m, c1, c3).is_none());
            // C2 and C3 are not, however
            assert_eq!(shortest_distance(&m, c2, c3).unwrap(), 1);
            // Can still get distance between terminal C and one of its Hs
            assert_eq!(shortest_distance(&m, c1, h1).unwrap(), 1);
            // Between central C and one of C1's Hs also fails though
            assert!(shortest_distance(&m, c2, h1).is_none());
            // Likewise between hydrogens at either end
            assert!(shortest_distance(&m, h1, h10).is_none());
            Ok(())
        }

        #[test]
        fn cyclic() -> Result<(), Error> {
            let mut m: AtomMap = ron::from_str(CYCLOHEXANE).unwrap();
            let carbons: Vec<AnyAtomlike> = m
                .iter::<Atom>()
                .filter(|&a| m.view(a).unwrap().element() == Element::C)
                .map(|a| a.as_atomlike())
                .collect();
            // C-C distances
            assert_eq!(shortest_distance(&m, carbons[0], carbons[1]).unwrap(), 1); // Adjacent C
            assert_eq!(shortest_distance(&m, carbons[0], carbons[2]).unwrap(), 2);
            assert_eq!(shortest_distance(&m, carbons[0], carbons[3]).unwrap(), 3); // Opposite
            assert_eq!(shortest_distance(&m, carbons[0], carbons[4]).unwrap(), 2);
            assert_eq!(shortest_distance(&m, carbons[0], carbons[5]).unwrap(), 1); // Adjacent again
            // C-H distances
            assert_eq!(
                shortest_distance(&m, carbons[0], "a2".parse::<Atom>()?).unwrap(), // H on that C
                1
            );
            assert_eq!(
                shortest_distance(&m, carbons[0], "a11".parse::<Atom>()?).unwrap(), // H on far side
                4
            );
            // H-H distances
            assert_eq!(
                shortest_distance(&m, "a2".parse::<Atom>()?, "a3".parse::<Atom>()?).unwrap(), // Geminal
                2
            );
            assert_eq!(
                shortest_distance(&m, "a2".parse::<Atom>()?, "a5".parse::<Atom>()?).unwrap(), // Vicinal
                3
            );
            assert_eq!(
                shortest_distance(&m, "a2".parse::<Atom>()?, "a11".parse::<Atom>()?).unwrap(), // Hs on opposite Cs
                5
            );
            // If we break the C0-C1 bond, they are still connected but much greater distance
            m.delete_bond("b5".parse()?);
            assert_eq!(
                m.view::<Atom>(carbons[0].try_into()?)
                    .unwrap()
                    .bonds()
                    .len(),
                3
            );
            assert_eq!(shortest_distance(&m, carbons[0], carbons[1]).unwrap(), 5);
            Ok(())
        }
    }

    mod are_connected {
        use super::*;

        #[test]
        fn simple_propane() -> Result<(), Error> {
            let m: AtomMap = ron::from_str(PROPANE).unwrap();
            let h1: AnyAtomlike = "a2".parse()?;
            let c1: AnyAtomlike = "a1".parse()?;
            let c2: AnyAtomlike = "a5".parse()?;
            let c3: AnyAtomlike = "a8".parse()?;
            let h10: AnyAtomlike = "a11".parse()?;
            // An atom is connected to itself
            assert!(are_connected(&m, c1, c1).unwrap());
            // Check various other connections
            assert!(are_connected(&m, c1, c3).unwrap());
            assert!(are_connected(&m, c1, c2).unwrap());
            assert!(are_connected(&m, c1, h1).unwrap());
            assert!(are_connected(&m, c2, h1).unwrap());
            assert!(are_connected(&m, h1, "a3".parse::<Atom>()?).unwrap());
            assert!(are_connected(&m, h10, "a6".parse::<Atom>()?).unwrap());
            assert!(are_connected(&m, h1, h10).unwrap());
            Ok(())
        }

        #[test]
        fn disconnected_propane() -> Result<(), Error> {
            let mut m: AtomMap = ron::from_str(PROPANE).unwrap();
            let h1: AnyAtomlike = "a2".parse()?;
            let c1: AnyAtomlike = "a1".parse()?;
            let c2: AnyAtomlike = "a5".parse()?;
            let c3: AnyAtomlike = "a8".parse()?;
            let h10: AnyAtomlike = "a11".parse()?;
            // Delete the C1-C2 bond
            m.delete_bond("b9".parse()?);
            // Now repeat the tests above; should get some different results
            // An atom is still connnected to itself
            assert!(are_connected(&m, c1, c1).unwrap());
            // C1 and C2 are of course now disconnected
            assert!(!are_connected(&m, c1, c2).unwrap());
            // The two terminal carbons are also now disconnected
            assert!(!are_connected(&m, c1, c3).unwrap());
            // C2 and C3 are not, however
            assert!(are_connected(&m, c2, c3).unwrap());
            // Terminal C and its Hs are still connected
            assert!(are_connected(&m, c1, h1).unwrap());
            // Central C and one of C1's Hs are not
            assert!(!are_connected(&m, c2, h1).unwrap());
            // Likewise for hydrogens at either end
            assert!(!are_connected(&m, h1, h10).unwrap());
            Ok(())
        }

        #[test]
        fn unconnected_h2o() -> Result<(), Error> {
            // Two separate water molecules
            let mut m = AtomMap::new();
            let o1 = m.add_atom(Element::O);
            let o2 = m.add_atom(Element::O);
            for o in [o1, o2] {
                for _ in 0..2 {
                    let h = m.add_atom(Element::H);
                    m.add_bond_unchecked(BondType::Covalent { order: 1.0 }, o, h);
                }
            }
            assert!(!are_connected(&m, o1, o2)?);
            Ok(())
        }

        #[test]
        fn h_bond_doesnt_connect_h2o() -> Result<(), Error> {
            let mut m = AtomMap::new();
            // Two water molecules
            let (o1, o1_h_bonds) = m.add_atom_and_saturate(Element::O);
            let o1h1 = o1_h_bonds.first().unwrap().to_owned();
            let (o2, _bonds) = m.add_atom_and_saturate(Element::O);
            let h1 = m
                .view(o1h1)
                .unwrap()
                .partners()
                .into_iter()
                .find(|&x| x != o1.as_bondable()) // Find the bonding partner that isn't O1
                .unwrap();
            // Connect via a hydrogen bond
            m.add_bond_unchecked(BondType::Hydrogen, o2, h1);
            assert!(!are_connected(&m, o1, o2).unwrap());
            Ok(())
        }
    }
}
