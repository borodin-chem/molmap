// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Methods for molecular graph traversal.
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

use std::collections::HashSet;

use crate::{entities::*, error::*, graph::MolGraph};

/// The result of a traversal step.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Step {
    /// A previously unexplored edge has been explored and followed to a previously
    /// unexplored node that is _not_ a pendant node.
    ///
    /// The entered, newly visited node is now the current node, and subsequent steps
    /// will explore the entered node. The entered node is now partially explored, in
    /// that the incoming edge has just been explored, but at least one unexplored
    /// outbound edge remains.
    ///
    /// The left node may or may not now be fully explored, depending on whether the
    /// explored edge was the final remaining unexplored edge leaving that node; it is
    /// indicated by `left_fully_explored`. If `left_fully_explored` is `true`, the left
    /// node will not be visited again.
    Forward {
        left: AnyAtomlike,
        edge: Bond,
        entered: AnyAtomlike,
        left_fully_explored: bool,
    },
    /// A previously unexplored edge to a pendant node has been explored and the pendant
    /// node visited, but the parent node has remained the current node.
    ///
    /// Pendant nodes are nodes with degree one, meaning that there is only one edge
    /// leading in to or out of it: the one between it and the parent node. The pendant
    /// node is thus now fully explored.
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
        parent_fully_explored: bool,
    },
    /// A previously unexplored edge leading to a **previously visited** node has been
    /// explored, resulting in the closure of a cycle.
    ///
    /// The confluence node (the previously visited node and the destination of the
    /// edge) has not been revisited as part of this. The current node remains the same.
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
    CycleClosure {
        current: AnyAtomlike,
        edge: Bond,
        confluence: AnyAtomlike,
        current_fully_explored: bool,
        confluence_fully_explored: bool,
    },
    /// The traversal has back-tracked to a node that was already visited but is only
    /// partially explored.
    ///
    /// This means that:
    /// 1. the left node was **fully explored**
    /// 2. the left node will **not be visited again**
    /// 3. the left node was **not a pendant node**, as these are explored without ever
    ///    becoming the current node
    ///
    /// No nodes or edges have been explored in this step.
    Back {
        left: AnyAtomlike,
        reentered: AnyAtomlike,
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
    /////
    ///// No edges have been explored in this step.
    /////
    ///// This means that:
    ///// 1. the left node was **fully explored**
    ///// 2. the left node will **not be visited again**
    ///// 3. the left node was **not a pendant node**
    //NewRoot {
    //    left: AnyAtomlike,
    //    entered: AnyAtomlike,
    //},
}

#[derive(Clone, Debug)]
struct QueueItem {
    priority: u16,
    degree: u8,
    edge: Bond,
    dest: AnyAtomlike,
}

#[derive(Clone, Debug)]
struct StackItem {
    node: AnyAtomlike,
    queue: Vec<QueueItem>,
}

/// A depth-first search of the molecular graph in the form of an iterator over
/// traversal steps.
///
/// The nodes of the graph are all the atoms and pseudoatoms, and the edges are
/// all the strong bonds (covalent/dipolar, ionic, metallic).
///
/// The graph is traversed from the root until all connected nodes and edges
/// have been explored. As such, if the `MolGraph` contains multiple disconnected
/// networks, only a subset will be traversed.
///
/// Each iteration returns an instance of [`Step`]. Not every step explores a
/// new edge or node – some just result in a change of the current node.
///
/// At a given node, bonds to pendant (terminal) hydrogen atoms will be explored
/// first, followed by carbon atoms, then other atoms by ascending atomic number,
/// then pseudoatoms.
#[derive(Debug)]
pub struct DepthFirstSearch<'m> {
    graph: &'m MolGraph,
    /// All visited nodes, whether partially or fully explored.
    visited_nodes: HashSet<AnyAtomlike>,
    explored_edges: HashSet<Bond>,
    stack: Vec<StackItem>,
    current_node: AnyAtomlike,
}

impl<'m> DepthFirstSearch<'m> {
    /// Initializes a traversal of the graph from the indicated atom or pseudoatom.
    ///
    /// # Errors
    ///
    /// Fails if the graph does not contain any atoms or pseudoatoms at all, or if
    /// the root is not a member of the graph.
    pub fn new(graph: &'m MolGraph, root: impl Atomlike) -> MolMapResult<Self> {
        let root = root.as_atomlike();
        if !(match root.resolve() {
            ResolvedAtomlike::Atom(atom) => graph.contains(atom),
            ResolvedAtomlike::Pseudoatom(pseudoatom) => graph.contains(pseudoatom),
        }) {
            return Err(MolMapError::InvalidId(root.into()));
        }
        let mut traversal = Self {
            graph,
            visited_nodes: HashSet::with_capacity(graph.atoms.len() + graph.pseudoatoms.len()),
            explored_edges: HashSet::with_capacity(graph.bonds.len()),
            stack: Vec::with_capacity(graph.atoms.len() / 2), // Be generous but assume at least half of nodes/atoms are pendant
            current_node: root,
        };
        // Visit root, populate its queue so that we're ready to go
        traversal.visited_nodes.insert(root);
        traversal.create_queue_on_stack(root);
        Ok(traversal)
    }
}

/// Helper methods for the traversal.
impl<'m> DepthFirstSearch<'m> {
    /// Adds a queue to the stack for a node and populates it by discovering outgoing edges.
    ///
    /// Only strong bonds are considered edges, and previously explored edges are ignored.
    fn create_queue_on_stack(&mut self, node: AnyAtomlike) {
        let mut queue: Vec<QueueItem> = Vec::with_capacity(6);
        let outward_bonds = match node.resolve() {
            ResolvedAtomlike::Atom(atom) => &self.graph.data(atom).unwrap().bonds,
            ResolvedAtomlike::Pseudoatom(pseudoatom) => &self.graph.data(pseudoatom).unwrap().bonds,
        };
        // Add in reverse order so that when we the subsequent descending order sort the
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
            // First, need to work out what the other end of this bond is
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
            queue.push(QueueItem {
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
        queue.sort_by_key(|a| std::cmp::Reverse(a.priority));
        self.stack.push(StackItem { node, queue });
    }

    /// Makes a forward transition along the next edge in the queue.
    ///
    /// # Panics
    ///
    /// Panics if either the stack or queue are empty.
    fn step_forward(&mut self) -> Step {
        let queue = &mut self.stack.last_mut().unwrap().queue;
        let next = queue.pop().unwrap();
        let edge = next.edge;
        let entered = next.dest;
        self.explored_edges.insert(edge);
        // Need to remove the node we just left if its queue is now empty, so
        // that it doesn't interfere if we back-track further down the branch
        let left = self.current_node;
        let left_fully_explored = queue.is_empty();
        if left_fully_explored {
            self.stack.pop();
        }
        // We have moved to a completely unexplored, unvisited node
        self.current_node = entered;
        self.visited_nodes.insert(entered);
        self.create_queue_on_stack(entered);
        Step::Forward {
            left,
            edge,
            entered,
            left_fully_explored,
        }
    }

    /// Explores the next edge in the queue and the destination node in the
    /// knowledge that the node is a pendant node, visiting it and fully exploring
    /// it, while keeping the current node (the parent node) the same.
    ///
    /// # Panics
    ///
    /// Panics if either the stack or queue are empty.
    fn explore_pendant(&mut self) -> Step {
        let parent = self.current_node;
        let parent_queue = &mut self.stack.last_mut().unwrap().queue;
        let next = parent_queue.pop().unwrap();
        let edge = next.edge;
        let pendant = next.dest;
        self.explored_edges.insert(edge);
        // Pendant node gets visited now, but it never becomes the current node!
        self.visited_nodes.insert(pendant);
        // Don't bother doing anything to the stack, it will get sorted in the
        // next step if it needs to be
        Step::Pendant {
            parent,
            edge,
            pendant,
            parent_fully_explored: parent_queue.is_empty(),
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
        let current_queue = &mut self.stack.last_mut().unwrap().queue;
        let next = current_queue.pop().unwrap();
        let edge = next.edge;
        let confluence = next.dest;
        self.explored_edges.insert(edge);
        // Look at the current node first (so that current_queue can be dropped and
        // we can reborrow from self) - does it have remaining edges to explore?
        let current_fully_explored = current_queue.is_empty();
        // We have just explored one of the edges that the confluence had in its queue,
        // so we should find its item in the stack and remove the edge from its queue
        let (_stack_pos, confluence_on_stack) = self
            .stack
            .iter_mut()
            .enumerate()
            .rev()
            .find(|x| x.1.node == confluence)
            .expect(
                "Must be on stack, as the edge we followed to get to it was unexplored until now",
            );
        if let Some(queue_pos) = confluence_on_stack
            .queue
            .iter()
            .position(|x| x.edge == edge)
        {
            confluence_on_stack.queue.remove(queue_pos);
        }
        let confluence_fully_explored = confluence_on_stack.queue.is_empty();
        // Back-tracking can cope with empty queues on the stack, so while we could
        // remove it from the stack if it is now empty, it would be a bit inefficient,
        // as it would cause a big shift on the stack
        //if confluence_fully_explored {
        //    self.stack.remove(stack_pos);
        //}
        Step::CycleClosure {
            current: self.current_node,
            edge,
            confluence,
            current_fully_explored,
            confluence_fully_explored,
        }
    }

    /// Back-tracks along the stack to the last unexhausted node, or returns `None`
    /// if none remain and the stack is now empty.
    ///
    /// Nodes with empty queues are removed from the stack.
    fn backtrack(&mut self) -> Option<Step> {
        while let Some(last) = self.stack.last() {
            if last.queue.is_empty() {
                self.stack.pop();
            } else {
                // Found a node that is only partially explored
                let left = self.current_node;
                let reentered = last.node;
                self.current_node = reentered;
                return Some(Step::Back { left, reentered });
            }
        }
        None
    }
}

impl<'m> Iterator for DepthFirstSearch<'m> {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        // Possible variants of Step and the scenarios in which they will be returned:
        // - Forward      - next edge to explore leads to unvisited node with degree > 1
        // - Pendant      - next edge to explore leads to unvisited node with degree == 1
        // - CycleClosure - next edge to explore leads to already visited node, a cycle is closed
        // - Back         - no remaining unexplored edges from node, but stack isn't empty
        //
        // If the stack is empty, or a backtrack fails to find any nodes with edges left to
        // explore, traversal is finished.

        if self.stack.is_empty() {
            None
        } else {
            // Last item in the stack should in all cases correspond to the current node
            debug_assert!(self.stack.last().unwrap().node == self.current_node);
            let current_queue = &self.stack.last().unwrap().queue;
            if current_queue.is_empty() {
                // Nothing in the queue, so the current node is fully explored -
                // backtrack and see if we can find one that isn't yet
                self.backtrack()
            } else {
                // The current node has at least one edge to explore, and each of
                // the helper methods we call will explore the next in the queue
                let next = current_queue.last().unwrap();
                // The edge should not have been previously explored
                debug_assert!(!self.explored_edges.contains(&next.edge));
                // Possible scenarios: Forward, Pendant, and CycleClosure
                // Cycle closure is distinguished by the fact that the node led
                // to by the edge is one that has already been visited
                let step = if self.visited_nodes.contains(&next.dest) {
                    self.close_cycle()
                } else {
                    // Is the destination node a pendant? If so, we will just visit it
                    // and return to the current node, effectively not leaving this one.
                    // Handily, we have stored the degree of the destination in the queue
                    if next.degree == 1 {
                        self.explore_pendant()
                    } else {
                        self.step_forward()
                    }
                };
                Some(step)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BondType, Element};

    use super::*;

    #[test]
    fn invalid_root() {
        let mut g = MolGraph::new();
        let h1 = g.add_atom(Element::H);
        g.delete_atom(h1);
        let h2 = g.add_atom(Element::H);
        let h3 = g.add_atom(Element::H);
        g.add_bond(BondType::Covalent { order: 1.0 }, h2, h3);
        assert!(DepthFirstSearch::new(&g, h1).is_err());
    }

    #[test]
    fn linear() {
        // Traverse ethane, starting at one of the hydrogen atoms
        let mut g = MolGraph::new();
        let c1 = g.add_atom(Element::C);
        let h1 = g.add_atom(Element::H);
        let h2 = g.add_atom(Element::H);
        let h3 = g.add_atom(Element::H);
        let c1h1 = g.add_bond(BondType::Covalent { order: 1.0 }, c1, h1);
        let c1h2 = g.add_bond(BondType::Covalent { order: 1.0 }, c1, h2);
        let c1h3 = g.add_bond(BondType::Covalent { order: 1.0 }, c1, h3);
        let c2 = g.add_atom(Element::C);
        let h4 = g.add_atom(Element::H);
        let h5 = g.add_atom(Element::H);
        let h6 = g.add_atom(Element::H);
        let c2h4 = g.add_bond(BondType::Covalent { order: 1.0 }, c2, h4);
        let c2h5 = g.add_bond(BondType::Covalent { order: 1.0 }, c2, h5);
        let c2h6 = g.add_bond(BondType::Covalent { order: 1.0 }, c2, h6);
        let c1c2 = g.add_bond(BondType::Covalent { order: 1.0 }, c1, c2);
        let mut traversal = DepthFirstSearch::new(&g, h1).unwrap();
        assert_eq!(
            traversal.next(),
            Some(Step::Forward {
                left: h1.into(),
                edge: c1h1,
                entered: c1.into(),
                left_fully_explored: true
            })
        );
        // Pendant Hs should be explored first, in insertion order
        assert_eq!(
            traversal.next(),
            Some(Step::Pendant {
                parent: c1.into(),
                edge: c1h2,
                pendant: h2.into(),
                parent_fully_explored: false
            })
        );
        assert_eq!(
            traversal.next(),
            Some(Step::Pendant {
                parent: c1.into(),
                edge: c1h3,
                pendant: h3.into(),
                parent_fully_explored: false
            })
        );
        // Then we move to the next carbon in the chain
        assert_eq!(
            traversal.next(),
            Some(Step::Forward {
                left: c1.into(),
                edge: c1c2,
                entered: c2.into(),
                left_fully_explored: true
            })
        );
        // Then the other H atoms, again in insertion order
        assert_eq!(
            traversal.next(),
            Some(Step::Pendant {
                parent: c2.into(),
                edge: c2h4,
                pendant: h4.into(),
                parent_fully_explored: false
            })
        );
        assert_eq!(
            traversal.next(),
            Some(Step::Pendant {
                parent: c2.into(),
                edge: c2h5,
                pendant: h5.into(),
                parent_fully_explored: false
            })
        );
        assert_eq!(
            traversal.next(),
            Some(Step::Pendant {
                parent: c2.into(),
                edge: c2h6,
                pendant: h6.into(),
                parent_fully_explored: true
            })
        );
        // Nothing left to traverse
        assert!(traversal.next().is_none());
    }

    #[test]
    fn branched() {
        // Traverse propane, starting at the carbon in the middle of the chain
        let mut g = MolGraph::new();
        let c1 = g.add_atom(Element::C);
        for _ in 0..3 {
            let h = g.add_atom(Element::H);
            g.add_bond(BondType::Covalent { order: 1.0 }, c1, h);
        }
        let c2 = g.add_atom(Element::C);
        for _ in 0..2 {
            let h = g.add_atom(Element::H);
            g.add_bond(BondType::Covalent { order: 1.0 }, c2, h);
        }
        let c3 = g.add_atom(Element::C);
        for _ in 0..3 {
            let h = g.add_atom(Element::H);
            g.add_bond(BondType::Covalent { order: 1.0 }, c3, h);
        }
        g.add_bond(BondType::Covalent { order: 1.0 }, c1, c2);
        g.add_bond(BondType::Covalent { order: 1.0 }, c2, c3);
        let traversal = DepthFirstSearch::new(&g, c2).unwrap();
        let mut explored_edges: Vec<Bond> = Vec::new();
        let mut visited_nodes: Vec<AnyAtomlike> = vec![c2.as_atomlike()];
        let mut backtracks: Vec<usize> = Vec::new();
        let mut cycle_closures: Vec<usize> = Vec::new();
        for (i, step) in traversal.enumerate() {
            match step {
                Step::Forward { edge, entered, .. } => {
                    explored_edges.push(edge);
                    visited_nodes.push(entered);
                }
                Step::Pendant { edge, pendant, .. } => {
                    explored_edges.push(edge);
                    visited_nodes.push(pendant);
                }
                Step::CycleClosure { edge, .. } => {
                    explored_edges.push(edge);
                    cycle_closures.push(i);
                }
                Step::Back { .. } => backtracks.push(i),
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
            visited_nodes
                .iter()
                .position(|&x| x == c1.as_atomlike())
                .unwrap()
                < visited_nodes
                    .iter()
                    .position(|&x| x == c3.as_atomlike())
                    .unwrap()
        );
    }

    #[test]
    fn cyclic() {
        // Traverse cyclohexane
        let mut g = MolGraph::new();
        let mut carbons: Vec<AnyAtomlike> = Vec::new();
        for n in 0..6 {
            let c = g.add_atom(Element::C);
            for _ in 0..2 {
                let h = g.add_atom(Element::H);
                g.add_bond(BondType::Covalent { order: 1.0 }, c, h);
            }
            if n != 0 {
                g.add_bond(BondType::Covalent { order: 1.0 }, c, carbons[n - 1]);
            }
            carbons.push(c.into());
        }
        g.add_bond(BondType::Covalent { order: 1.0 }, carbons[0], carbons[5]);
        let traversal = DepthFirstSearch::new(&g, carbons[0]).unwrap();
        let mut explored_edges: Vec<Bond> = Vec::new();
        let mut visited_nodes: Vec<AnyAtomlike> = vec![carbons[0]];
        let mut backtracks: Vec<usize> = Vec::new();
        let mut cycle_closures: Vec<usize> = Vec::new();
        for (i, step) in traversal.enumerate() {
            match step {
                Step::Forward { edge, entered, .. } => {
                    explored_edges.push(edge);
                    visited_nodes.push(entered);
                }
                Step::Pendant { edge, pendant, .. } => {
                    explored_edges.push(edge);
                    visited_nodes.push(pendant);
                }
                Step::CycleClosure { edge, .. } => {
                    explored_edges.push(edge);
                    cycle_closures.push(i);
                }
                Step::Back { .. } => backtracks.push(i),
            }
        }
        assert_eq!(explored_edges.len(), 18);
        assert_eq!(visited_nodes.len(), 18);
        // As search is depth-first and pendant H is explored before the carbon chain,
        // the ring should only have been closed in the very last step
        // No backtracking occurs so total steps = 5 forward + (6 * 2 pendant) + 1 cycle-closing
        assert_eq!(cycle_closures, [17])
    }

    #[test]
    fn disconnected() {
        // Ensure that only the connected network is traversed
        let mut g = MolGraph::new();
        // Two water molecules
        let o1 = g.add_atom(Element::O);
        let o2 = g.add_atom(Element::O);
        for o in [o1, o2] {
            for _ in 0..2 {
                let h = g.add_atom(Element::H);
                g.add_bond(BondType::Covalent { order: 1.0 }, o, h);
            }
        }
        let mut traversal = DepthFirstSearch::new(&g, o1).unwrap();
        for _ in &mut traversal {}
        // Check that O of second water molecule was never visited
        assert!(!traversal.visited_nodes.contains(&o2.as_atomlike()));
    }

    #[test]
    fn h_bond_doesnt_connect() {
        let mut g = MolGraph::new();
        // Two water molecules
        let o1 = g.add_atom(Element::O);
        let o2 = g.add_atom(Element::O);
        let mut h1: Option<Atom> = None;
        for o in [o1, o2] {
            for _ in 0..2 {
                let h = g.add_atom(Element::H);
                if h1.is_none() {
                    h1 = Some(h);
                }
                g.add_bond(BondType::Covalent { order: 1.0 }, o, h);
            }
        }
        // Connect via a hydrogen bond
        let h1 = h1.unwrap();
        g.add_bond(BondType::Hydrogen, o2, h1);
        let mut traversal = DepthFirstSearch::new(&g, o1).unwrap();
        for _ in &mut traversal {}
        // Check that O of second water molecule was never visited
        assert!(!traversal.visited_nodes.contains(&o2.as_atomlike()));
    }
}
