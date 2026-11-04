// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla pub(crate)lic
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Implementation of breadth-first graph traversal.
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

#![expect(unused)]

use std::collections::{HashSet, VecDeque};

use crate::{
    atomic::AtomMap,
    entities::*,
    error::*,
    traits::{Map, Stores},
};

/// The result of a step in a breadth-first traversal.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum BFStep {
    /// A previously unexplored edge to a previously unvisited child node has been
    /// explored and the child node visited, but the parent node has remained the
    /// current node.
    ///
    /// The visited child node is not a pendant node, and nor has a cycle been closed.
    ///
    /// The parent node may or may not now be fully explored, depending on whether the
    /// explored edge was the final remaining unexplored edge leaving that node; it is
    /// indicated by `parent_fully_explored`.
    Child {
        parent: AnyAtomlike,
        edge: Bond,
        child: AnyAtomlike,
        child_depth: usize,
        parent_fully_explored: bool,
    },
    /// A previously unexplored edge to a previously unvisited child pendant node has
    /// been explored and the pendant node visited, but the parent node has remained the
    /// current node.
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
    Pendant {
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
    /// Prior to this step, the confluence node will have definitely been in the node
    /// queue, as it was only partially explored. That node may or may not now be fully
    /// explored, depending on whether the explored edge was its final remaining
    /// unexplored edge. The explored edge will now have been removed from the edge
    /// queue of the confluence node. If the confluence node's edge queue is now empty,
    /// it will not be visited or explored again, and `confluence_fully_explored` will
    /// be `true` to indicate as such.
    ///
    /// Similarly, the node from which the cycle-closing edge came may or may not now be
    /// fully explored; this is indicated by `current_fully_explored`.
    ///
    /// The confluence may be at the same depth as the current node, or it may be one
    /// level deeper; the depth of the confluence is given by `confluence_depth`.
    CycleClosure {
        current: AnyAtomlike,
        edge: Bond,
        confluence: AnyAtomlike,
        confluence_depth: usize,
        current_fully_explored: bool,
        confluence_fully_explored: bool,
    },
    /// The traversal has increased in depth by moving to a node that was already
    /// visited but is only partially explored.
    ///
    /// The entered node is not necessarily a child of the left one.
    ///
    /// This means that:
    /// 1. the left node was **fully explored**
    /// 2. the left node will **not be visited again**
    /// 3. the entered node is **not a pendant node**, as these are explored without ever
    ///    becoming the current node
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
    /// 3. the entered node is **not a pendant node**, as these are explored without ever
    ///    becoming the current node
    ///
    /// No nodes or edges have been explored in this step.
    SideTrack {
        left: AnyAtomlike,
        entered: AnyAtomlike,
        current_depth: usize,
    },
    // This NewRoot variant may be useful in future if we have traversals/iterators that
    // are allowed to jump to other networks after exhausting the initial one
    ///// A new, previously undiscovered node, unconnected to any previously discovered
    ///// node, has been selected, visited, and set as the new current node.
    //NewRoot {
    //    left: AnyAtomlike,
    //    entered: AnyAtomlike,
    //},
}

#[derive(Clone, Debug)]
struct EdgeQueueItem {
    priority: u16,
    degree: u8,
    edge: Bond,
    dest: AnyAtomlike,
}

#[derive(Clone, Debug)]
struct NodeQueueItem {
    node: AnyAtomlike,
    depth: usize,
    /// The queue of edges waiting to be explored.
    ///
    /// The queue should be sorted by their `priority` field in _descending numerical
    /// order_, which leaves the edges sorted by their _actual_ priority in _ascending_
    /// order. The next edge in the queue -- the one with the next-highest priority --
    /// is then obtained by popping it off the _back_ of the queue.
    edge_queue: Vec<EdgeQueueItem>,
}

/// A breadth-first search of the molecular graph in the form of an iterator over
/// traversal steps.
///
/// The nodes of the graph are all the atoms and pseudoatoms, and the edges are
/// all the strong bonds (covalent/dipolar, ionic, metallic).
///
/// The graph is traversed from the root until all connected nodes and edges
/// have been explored. As such, if the graph contains multiple disconnected
/// networks, only a subset will be traversed.
///
/// Each iteration returns an instance of [`BFStep`]. Not every step explores a
/// new edge or node – some just result in a change of the current node.
///
/// At a given node, bonds to pendant (terminal) hydrogen atoms will be explored
/// first, followed by carbon atoms, then other atoms by ascending atomic number,
/// then pseudoatoms.
#[derive(Debug)]
pub(crate) struct BreadthFirstSearch<'m> {
    graph: &'m AtomMap,
    /// All visited nodes, whether partially or fully explored.
    visited_nodes: HashSet<AnyAtomlike>,
    explored_edges: HashSet<Bond>,
    /// The queue of nodes waiting to be explored.
    ///
    /// The next node to be explored is taken from the front, and nodes are queued by
    /// pushing them onto the back.
    node_queue: VecDeque<NodeQueueItem>,
    current_node: AnyAtomlike,
    current_depth: usize,
}

impl<'m> BreadthFirstSearch<'m> {
    /// Initializes a traversal of the graph from the indicated atom or pseudoatom.
    ///
    /// # Errors
    ///
    /// Fails if the graph does not contain any atoms or pseudoatoms at all, or if
    /// the root is not a member of the graph.
    pub(crate) fn new(graph: &'m AtomMap, root: impl Atomlike) -> MolMapResult<Self> {
        let root = root.as_atomlike();
        if !(match root.resolve() {
            ResolvedAtomlike::Atom(atom) => graph.contains(atom),
            ResolvedAtomlike::Pseudoatom(pseudoatom) => graph.contains(pseudoatom),
        }) {
            return Err(MolMapError::InvalidId(root.into()));
        }
        let mut traversal = Self {
            graph,
            visited_nodes: HashSet::with_capacity(
                <AtomMap as Stores<Atom>>::count(graph)
                    + <AtomMap as Stores<Pseudoatom>>::count(graph),
            ),
            explored_edges: HashSet::with_capacity(<AtomMap as Stores<Bond>>::count(graph)),
            node_queue: VecDeque::with_capacity(<AtomMap as Stores<Atom>>::count(graph) / 2), // Be generous but assume at least half of nodes/atoms are pendant
            current_node: root,
            current_depth: 0,
        };
        // Visit root, populate its queue so that we're ready to go
        traversal.visited_nodes.insert(root);
        traversal.create_edge_queue_in_node_queue(root, 0);
        Ok(traversal)
    }
}

/// Helper methods for the traversal.
impl<'m> BreadthFirstSearch<'m> {
    /// Adds an edge queue to the node queue for a node and populates it by discovering
    /// outgoing edges.
    ///
    /// Only strong bonds are considered edges, and previously explored edges are ignored.
    fn create_edge_queue_in_node_queue(&mut self, node: AnyAtomlike, depth: usize) {
        let mut edge_queue: Vec<EdgeQueueItem> = Vec::with_capacity(6);
        let outward_bonds = match node.resolve() {
            ResolvedAtomlike::Atom(atom) => &self.graph.data(atom).unwrap().bonds,
            ResolvedAtomlike::Pseudoatom(pseudoatom) => &self.graph.data(pseudoatom).unwrap().bonds,
        };
        // Add in reverse order so that when we do the subsequent descending order sort the
        // original order is maintained between ties
        for &bond in outward_bonds.iter().rev() {
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
            // Later we will want to make it possible to use different sort strategies,
            // but for now we just prioritize first pendant (non-bridging) hydrogen atoms,
            // then carbon atoms, then other atoms in order of atomic number, then
            // pseudoatoms.
            // First, need to work out what the other end of this bond is.
            let dest_bondable = if bond_data.start == node.as_bondable() {
                bond_data.end
            } else if bond_data.end == node.as_bondable() {
                bond_data.start
            } else {
                unreachable!("Currently, the node will always be one of the two bonding partners")
            };
            let (dest, priority, degree) = match dest_bondable.resolve() {
                ResolvedBondable::Atom(atom) => {
                    let atom_data = self.graph.data(atom).unwrap();
                    let degree = atom_data
                        .bonds
                        .iter()
                        .filter(|&&b| self.graph.data(b).unwrap().bond_type.is_strong())
                        .count();
                    let priority: u16 = match atom_data.element.atomic_number() {
                        1 => {
                            if degree == 1 {
                                1 // Pendant H has highest priority
                            } else {
                                257 // Bridging H has priority according to its Z i.e. third-highest
                            }
                        }
                        6 => 6,              // Carbon has second-highest priority
                        x => x as u16 + 256, // Addition of 256 is a single bit flip 0 -> 1 with no carrying
                    };
                    (atom.as_atomlike(), priority, degree)
                }
                ResolvedBondable::Pseudoatom(pseudoatom) => {
                    let pseudoatom_data = self.graph.data(pseudoatom).unwrap();
                    let degree = pseudoatom_data
                        .bonds
                        .iter()
                        .filter(|&&b| self.graph.data(b).unwrap().bond_type.is_strong())
                        .count();
                    let priority = u16::MAX; // Easy to compare as larger
                    (pseudoatom.as_atomlike(), priority, degree)
                }
            };
            edge_queue.push(EdgeQueueItem {
                edge: bond,
                dest,
                priority,
                degree: degree.try_into().expect(
                    "Seems reasonable to assume that no node will have a degree of more than 255",
                ),
            })
        }
        // Lower number for priority means a higher priority
        // Sort queue by priority in descending numerical order, which means sorting by
        // ascending priority. That we can get the next edge in the queue - the one with
        // the next-highest priority - by popping off the back of the queue
        edge_queue.sort_by_key(|a| std::cmp::Reverse(a.priority));
        self.node_queue.push_back(NodeQueueItem {
            node,
            depth,
            edge_queue,
        });
    }

    /// Explores the next edge in the current node's edge queue and the destination node
    /// in the knowledge that the node is a child node, visiting it and adding an edge
    /// queue for it to the back of the node queue, while keeping the current node (the
    /// parent node) the same.
    ///
    /// This method relies on the destination node not having been previously visited.
    ///
    /// # Panics
    ///
    /// Panics if either the node queue or the current node's edge queue are empty.
    fn explore_child(&mut self) -> BFStep {
        let parent = self.current_node;
        let parent_queue = &mut self.node_queue.front_mut().unwrap().edge_queue;
        let next = parent_queue.pop().unwrap();
        let edge = next.edge;
        let child = next.dest;
        let child_depth = self.current_depth + 1;
        let parent_fully_explored = parent_queue.is_empty();
        self.explored_edges.insert(edge);
        // Child node gets visited now, but will become the current node later,
        // once all nodes at the current depth have been fully explored
        self.visited_nodes.insert(child);
        self.create_edge_queue_in_node_queue(child, child_depth);
        // Don't need to remove the node's item from the node queue, it will get
        // sorted in the next step if it needs to be
        BFStep::Child {
            parent,
            edge,
            child,
            child_depth,
            parent_fully_explored,
        }
    }

    /// Explores the next edge in the queue and the destination node in the
    /// knowledge that the node is a pendant node, visiting it and fully exploring
    /// it, while keeping the current node (the parent node) the same.
    ///
    /// # Panics
    ///
    /// Panics if either the node queue or the current node's edge queue are empty.
    fn explore_pendant(&mut self) -> BFStep {
        let parent = self.current_node;
        let parent_queue = &mut self.node_queue.front_mut().unwrap().edge_queue;
        let next = parent_queue.pop().unwrap();
        let edge = next.edge;
        let pendant = next.dest;
        let pendant_depth = self.current_depth + 1;
        let parent_fully_explored = parent_queue.is_empty();
        self.explored_edges.insert(edge);
        // Pendant node gets visited now, but it never becomes the current node!
        self.visited_nodes.insert(pendant);
        // Don't need to remove the node's item from the node queue, it will get
        // sorted in the next step if it needs to be
        BFStep::Pendant {
            parent,
            edge,
            pendant,
            pendant_depth,
            parent_fully_explored,
        }
    }

    /// Explores the next edge in the queue in the knowledge that the destination
    /// node is a previously visited "confluence" node, removing the explored edge
    /// from the queue of the confluence node on the stack, while keeping the
    /// current node the same.
    ///
    /// This method relies on the destination node having been previously visited.
    ///
    /// # Panics
    ///
    /// Panics if either the node queue or the current node's edge queue are empty,
    /// or if the confluence node does not have an item waiting in the node queue.
    fn close_cycle(&mut self) -> BFStep {
        let current_queue = &mut self.node_queue.front_mut().unwrap().edge_queue;
        let next = current_queue.pop().unwrap();
        let edge = next.edge;
        let confluence = next.dest;
        self.explored_edges.insert(edge);
        // Destination is already visited, don't need to visit it again!
        // Now look at the current node (first, so that current_queue can be dropped and
        // we can reborrow from self) - does it have remaining edges to explore?
        let current_fully_explored = current_queue.is_empty();
        // We have just explored one of the edges that the confluence had in its queue,
        // so we should find its item in the node queue and remove this edge from its edge queue
        let (_node_queue_pos, confluence_in_node_queue) = self
            .node_queue
            .iter_mut()
            .enumerate()
            .find(|x| x.1.node == confluence)
            .expect(
                "Must be in node queue, as the edge we followed to get to it was unexplored until now",
            );
        if let Some(edge_queue_pos) = confluence_in_node_queue
            .edge_queue
            .iter()
            .position(|x| x.edge == edge)
        {
            confluence_in_node_queue.edge_queue.remove(edge_queue_pos);
        }
        let confluence_depth = confluence_in_node_queue.depth;
        let confluence_fully_explored = confluence_in_node_queue.edge_queue.is_empty();
        // Forward/side-tracking can cope with empty items in the node queue, so while
        // we could remove it now if it is now empty, it would be a bit inefficient,
        // as it would open a gap and therefore cause half the queue to shift
        BFStep::CycleClosure {
            current: self.current_node,
            edge,
            confluence,
            confluence_depth,
            current_fully_explored,
            confluence_fully_explored,
        }
    }

    /// Tracks along the node queue to the next unexhausted node, or returns `None`
    /// if none remain and the node queue is now empty.
    ///
    /// If there are remaining unexhausted nodes at the same depth, this results in a
    /// "side"-track (the current depth remains the same) and `BFStep::SideTrack` is
    /// returned, while if the next unexhausted node is a level deeper, the result is a
    /// "forward"-track and `BFStep::ForwardTrack` is returned.
    ///
    /// Any encountered node queue items with no remaining unexplored edges are removed
    /// from the node queue.
    fn track(&mut self) -> Option<BFStep> {
        while let Some(front) = self.node_queue.front() {
            if front.edge_queue.is_empty() {
                self.node_queue.pop_front();
            } else {
                // Found a node that is only partially explored
                let left = self.current_node;
                let entered = front.node;
                self.current_node = entered;
                let current_depth = self.current_depth;
                let new_depth = front.depth;
                let step = if new_depth == current_depth {
                    BFStep::SideTrack {
                        left,
                        entered,
                        current_depth,
                    }
                } else {
                    debug_assert_eq!(new_depth, current_depth + 1);
                    self.current_depth = new_depth;
                    BFStep::ForwardTrack {
                        left,
                        entered,
                        new_depth,
                    }
                };
                return Some(step);
            }
        }
        // No unexhausted nodes encountered
        None
    }
}

impl<'m> Iterator for BreadthFirstSearch<'m> {
    type Item = BFStep;

    fn next(&mut self) -> Option<Self::Item> {
        // Possible variants of BFStep and the scenarios in which they will be returned:
        // - Child        - next edge to explore leads to unvisited node with degree > 1
        // - Pendant      - next edge to explore leads to unvisited node with degree == 1
        // - CycleClosure - next edge to explore leads to already visited node, a cycle is closed
        // - ForwardTrack - no remaining unexplored edges from node, but queue isn't empty
        // - SideTrack    - no remaining unexplored edges from node, but queue isn't empty
        //
        // If the queue is empty, or a forward/sidetrack fails to find any nodes with edges
        // left to explore, traversal is finished.

        if self.node_queue.is_empty() {
            None
        } else {
            // Next node in the queue should in all cases correspond to the current node
            debug_assert!(self.node_queue.front().unwrap().node == self.current_node);
            let current_edge_queue = &self.node_queue.front().unwrap().edge_queue;
            if current_edge_queue.is_empty() {
                // Nothing in the queue, so the current node is fully explored -
                // track along queue and see if we can find one that isn't yet
                self.track()
            } else {
                // The current node has at least one edge to explore, and each of
                // the helper methods we call will explore the next edge in the queue
                let next = current_edge_queue.last().unwrap();
                // The edge should not have been previously explored
                debug_assert!(!self.explored_edges.contains(&next.edge));
                // Possible scenarios: Child, Pendant, and CycleClosure
                // Cycle closure is distinguished by the fact that the node led
                // to by the edge is one that has already been visited
                let step = if self.visited_nodes.contains(&next.dest) {
                    self.close_cycle()
                } else {
                    // Is the destination node a pendant?
                    // Handily, we have stored the degree of the destination in the queue
                    if next.degree == 1 {
                        self.explore_pendant()
                    } else {
                        self.explore_child()
                    }
                };
                Some(step)
            }
        }
    }
}
