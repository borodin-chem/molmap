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

use std::{cmp::Ordering, collections::HashSet};

use crate::{
    Element,
    categories::*,
    entities::*,
    error::*,
    graph::{MolGraph, keys::Keyed},
};

/// The result of a traversal step.
#[derive(Copy, Clone, Debug)]
pub enum Step {
    /// No step has been made.
    ///
    /// Returned to indicate the traversal root before commencing exploration.
    ///
    /// The root node has been visited, the outgoing edges have been discovered,
    /// and a queue on the traversal stack has been populated, but no exploration
    /// has yet taken place.
    Initial { root: AnyAtomlike },
    /// A new, previously undiscovered node, unconnected to any previously discovered
    /// node, has been selected, visited, and set as the new current node.
    ///
    /// The entered node has not been explored at all, and subsequent steps will explore
    /// the entered node; the exception to this is if the entered node is an isolated
    /// node (it has degree zero and no edges leading in or out of it), in which case
    /// the entered node is automatically immediately fully explored.
    ///
    /// No edges have been explored in this step.
    ///
    /// This means that:
    /// 1. the left node was **fully explored**
    /// 2. the left node will **not be visited again**
    /// 3. the left node was **not a pendant node**, as these are explored without ever
    ///    being visited (becoming the current node) and no back-tracking occurs (the
    ///    parent remains the current node)
    MoveUnconnected {
        left: AnyAtomlike,
        entered: AnyAtomlike,
    },
    /// The traversal has back-tracked to a node that was already visited but is only
    /// partially explored.
    ///
    /// This means that:
    /// 1. the left node was **fully explored**
    /// 2. the left node will **not be visited again**
    /// 3. the left node was **not a pendant node**, as these are explored without ever
    ///    being visited (becoming the current node) and no back-tracking occurs (the
    ///    parent remains the current node)
    ///
    /// No nodes or edges have been discovered or explored in this step.
    Back {
        left: AnyAtomlike,
        entered: AnyAtomlike,
    },
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
    /// A previously unexplored edge leading to a **previously visited** node has been
    /// explored, resulting in the closure of a cycle.
    ///
    /// The confluence node (the previously visited node and the destination of the
    /// edge) has not been visited as part of this. The current node remains the same.
    ///
    /// Prior to this step, the confluence node will have definitely been on the stack,
    /// as it was only partially explored. That node may or may not now be fully
    /// explored, depending on whether the explored edge was its final remaining
    /// unexplored edge. The explored edge will now have been removed from the queue of
    /// the confluence node on the stack, and the queue will have been removed from the
    /// stack if it was then empty. If that was the case, the confluence node will not
    /// be visited or explored again, and `confluence_fully_explored` will be `true` to
    /// indicate as such.
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
    /// A previously unexplored edge and pendant node have been explored without being
    /// visited, such that the parent node has remained the current node.
    ///
    /// Pendant nodes are nodes with degree one, meaning that there is only one edge
    /// leading in to or out of it: the one between it and the parent node.
    ///
    /// A pendant node is most often a hydrogen atom, but may also be e.g. a halogen,
    /// =O, ≡N etc.
    ///
    /// This means that:
    /// 1. the pendant node was **fully explored**
    /// 2. the pendant node will **never be visited**
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
    /// No new nodes or edges have been discovered, visited, or explored, because all
    /// possibilities for traversal have been exhausted. No further traversal will occur
    /// and all future steps will be of this variant, or the iterator will return `None`.
    NoPossible { last: AnyAtomlike },
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

impl StackItem {
    fn new(node: AnyAtomlike) -> Self {
        Self {
            node,
            queue: Vec::with_capacity(6),
        }
    }
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
/// new edge or node – some just result in a change of the current node, and the
/// first step returned will always be [`Step::Initial`] to indicate the starting
/// node.
///
/// At a given node, bonds to pendant hydrogen atoms will be explored before
/// other outgoing bonds.
#[derive(Debug)]
pub struct DepthFirstSearch<'m> {
    graph: &'m MolGraph,
    /// All visited nodes, whether partially or fully explored.
    visited_nodes: HashSet<AnyAtomlike>,
    explored_edges: HashSet<Bond>,
    stack: Vec<StackItem>,
    current_node: AnyAtomlike,
    /// Used to indicate whether `current_node` was unvisited prior to this step, in
    /// which case it requires a queue to be created for it on the stack, for its
    /// outgoing edges to be discovered, and for it to be added to `visited_nodes`.
    new_visit: bool,
    prev_step: Step,
}

impl<'m> DepthFirstSearch<'m> {
    /// Initializes a traversal of the graph from the indicated atom or pseudoatom.
    ///
    /// # Errors
    ///
    /// Fails if the graph does not contain any atoms or pseudoatoms at all, or if
    /// the root is not a member of the graph.
    pub fn new(graph: &'m MolGraph, root: impl Atomlike) -> MolMapResult<Self> {
        let atomlike_count = graph.atoms.len() + graph.pseudoatoms.len();
        if atomlike_count == 0 {
            Err(MolMapError::EmptyMap)
        } else {
            Ok(Self {
                graph,
                visited_nodes: HashSet::with_capacity(atomlike_count),
                explored_edges: HashSet::with_capacity(graph.bonds.len()),
                stack: Vec::with_capacity(atomlike_count / 2), // Be generous but assume at least half of nodes/atoms are pendant
                current_node: root.as_atomlike(),
                new_visit: true,
                prev_step: Step::Initial {
                    root: root.as_atomlike(),
                },
            })
        }
    }
}

impl<'m> Iterator for DepthFirstSearch<'m> {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        // First check if we've started traversal at all
        match self.prev_step {
            Step::NoPossible { last } => return None, // No further iteration, traversal complete
            _ => (),
        }

        // Possible variants of Step and the scenarios in which they will be returned:
        //
        // 0. MoveUnconnected   - never returned by `next`, as traversal halts once connected network is exhausted
        //
        // 1. Initial           - nothing has been done yet - tell the user where the root is
        //                      - indicated by `prev_step` being `Initial` but `new_visit` being `true`
        //
        // 2. NoPossible        - no remaining unexplored edges from node, and stack is empty - halt traversal
        // 3. Back              - no remaining unexplored edges from node, but stack isn't empty - back-track
        //
        // 4. CycleClosure      - next edge to explore leads to explored node, a cycle is closed
        //
        // 5. Forward           - next edge to explore leads to unexplored node with degree > 1
        // 6. Pendant           - next edge to explore leads to unexplored node with degree == 1

        // Is the node being visited for the first time and hasn't been explored at all yet?
        if self.new_visit {
            // Node needs to have its queue populated with unexplored outgoing edges,
            // but there is currently no corresponding item on the stack for it
            self.visited_nodes.insert(self.current_node);
            self.stack.push(StackItem::new(self.current_node));
        }

        // Last item in the stack should now in all cases correspond to the current node
        let Some(node_on_stack) = self.stack.last_mut() else {
            // But if the stack is completely empty then clearly we have nothing left to
            // do and it's just that we already dropped the node's emptied queue
            let step = Step::NoPossible {
                last: self.current_node,
            };
            self.prev_step = step;
            return Some(step);
        };
        debug_assert!(node_on_stack.node == self.current_node);

        // Now actually populate the queue (for newly visited nodes, that is -
        // previously visited nodes will still have their queue on the stack)
        if self.new_visit {
            let outward_bonds = match self.current_node.resolve() {
                ResolvedAtomlike::Atom(atom) => &self.graph.data(atom).unwrap().bonds,
                ResolvedAtomlike::Pseudoatom(pseudoatom) => {
                    &self.graph.data(pseudoatom).unwrap().bonds
                }
            };
            let mut bond_sort: Vec<(isize, QueueItem)> = Vec::with_capacity(6);
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
                // Later we will want to make it possible to use different sort strategies,
                // but for now we just prioritize first pendant (non-bridging) hydrogen atoms,
                // then carbon atoms, then other atoms in order of atomic number, then
                // pseudoatoms.
                // First, need to work out what the other end of this bond is
                let dest = if bond_data.start != self.current_node.as_bondable() {
                    bond_data.start
                } else {
                    bond_data.end
                };
                match dest.resolve() {
                    ResolvedBondable::Atom(atom) => {
                        let atom_data = self.graph.data(atom).unwrap();
                        let priority: u16 = match atom_data.element.atomic_number() {
                            1 => {
                                if atom_data.bonds == [bond] {
                                    1
                                } else {
                                    257
                                }
                            }
                            12 => 12,
                            x => x as u16 + 256, // Single bit flip
                        };
                        node_on_stack.queue.push(QueueItem {
                            edge: bond,
                            dest: atom.into(),
                            priority: priority,
                            degree: atom_data.bonds.len().try_into().expect("Seems reasonable to assume that no node will have a degree of more than 255"),
                        })
                    }
                    ResolvedBondable::Pseudoatom(pseudoatom) => {
                        node_on_stack.queue.push(QueueItem {
                            edge: bond,
                            dest: pseudoatom.into(),
                            priority: u16::MAX, // Easy to compare as larger
                            degree: 1,          // This may not be true later but will do for now
                        })
                    }
                }
            }
            // Sort queue by priority in descending numerical order, which means
            // ascending priority, so that we can pop off the back
            node_on_stack
                .queue
                .sort_by(|a, b| b.priority.cmp(&a.priority));

            // Even if this is still the current node after this step, it will no
            // longer be a "fresh" visit (even if we didn't go anywhere in the
            // meantime and it is still technically the "first" visit)
            self.new_visit = false;

            // Finally, deal with the case where this is the initial step of the
            // whole traversal by returning Step::Initial early, as promised,
            // without actually exploring any of the just-discovered edges yet
            match self.prev_step {
                Step::Initial { root: initial } => {
                    // Node was already added to visited_nodes because new_visit was true
                    // Stack item has been added and the queue populated
                    // current_node is already set to the root from when the traverser was constructed
                    // We just set new_visit to false, and prev_step stays the same
                    return Some(Step::Initial { root: initial });
                }
                _ => (),
            }
        }

        // Take the next edge from the end of the queue - or at least, try to
        let Some(next) = node_on_stack.queue.pop() else {
            // There _wasn't_ anything in the queue, so the current node is fully
            // explored - remove its queue from the stack, and we are looking at
            // either NoPossible or Back
            self.stack.pop();
            // Distinguish between the two scenarios - is the stack now empty,
            // or can we do a back-track?
            let step = if self.stack.is_empty() {
                // Clearly we're done traversing entirely!
                // Don't bother changing the state, as next iteration will immediately
                // return None when it sees this was the previous step
                let step = Step::NoPossible {
                    last: self.current_node,
                };
                self.prev_step = step;
                return Some(step);
            } else {
                // Back-track along the stack to the last unexhausted node
                // Since we remove items from the stack once their queue is empty,
                // there should be no exhausted ones on the stack, so whatever comes
                // off next should have a non-empty queue
                let revisited = self
                    .stack
                    .last()
                    .expect("We checked the stack's not empty already");
                let left = self.current_node;
                let entered = revisited.node;
                self.current_node = entered;
                assert!(!self.new_visit); // After back-tracking the current node will always be explored
                let step = Step::Back { left, entered };
                self.prev_step = step;
                return Some(step);
            };
        };

        // If we have reached here, the current node has at least one edge to explore!
        // Possible scenarios: Forward, Pendant, and CycleClosure
        // In all cases we explore the edge we've taken from the queue
        debug_assert!(!self.explored_edges.contains(&next.edge));
        self.explored_edges.insert(next.edge);

        // Cycle closure is distinguished by the fact that the node led
        // to by the edge is one that has already been visited
        if self.visited_nodes.contains(&next.dest) {
            let edge = next.edge;
            let confluence = next.dest;
            // Quite a bit to sort out on the stack
            // Look at the current node first (so that node_on_stack can be dropped and
            // we can reborrow it) - does it have remaining edges to explore?
            let current_fully_explored = node_on_stack.queue.is_empty();
            // We have just explored one of the edges that the confluence had in its queue,
            // so we should find its item in the stack and remove the edge from its queue
            let (stack_pos, confluence_on_stack) = self
                .stack
                .iter_mut()
                .enumerate()
                .rev()
                .find(|x| x.1.node == confluence)
                .expect("Must be on stack, as the edge we followed to get to it was unexplored until now");
            if let Some(queue_pos) = confluence_on_stack
                .queue
                .iter()
                .position(|x| x.edge == edge)
            {
                confluence_on_stack.queue.remove(queue_pos);
            }
            // If the queue is now empty, it should be removed from the stack
            let confluence_fully_explored = confluence_on_stack.queue.is_empty();
            if confluence_fully_explored {
                self.stack.remove(stack_pos);
                // This is a little inefficient, and maybe it would be better if back-tracking
                // were changed to cope with empty queues on the stack (currently it relies
                // on them all being non-empty)
            }
            let step = Step::CycleClosure {
                current: self.current_node,
                edge,
                confluence,
                current_fully_explored,
                confluence_fully_explored,
            };
            self.prev_step = step;
            return Some(step);
        }

        // Now we are left with only two scenarios, Pendant and Forward
        // Key question: is the destination node a pendant? Will we continue on
        // from it or will we end up back here?
        // This is distinguished by the degree of the destination node, which,
        // handily, we have stored for each edge in the queue
        if next.degree == 1 {
            // Destination node is a pendant node - it gets visited now, and it
            // never becomes the current node!
            self.visited_nodes.insert(next.dest);
            // Don't bother doing anything to the stack, it will get sorted in the
            // next step if it needs to be
            let step = Step::Pendant {
                parent: self.current_node,
                edge: next.edge,
                pendant: next.dest,
                parent_fully_explored: node_on_stack.queue.is_empty(),
            };
            self.prev_step = step;
            Some(step)
        } else {
            // Don't need to visit destination node now, it will be visited in next step
            let left = self.current_node;
            let edge = next.edge;
            let entered = next.dest;
            // Need to remove the node we just left if its queue is now empty, so
            // that it doesn't interfere if we back-track further down the branch
            let left_fully_explored = node_on_stack.queue.is_empty();
            if left_fully_explored {
                self.stack.pop();
            }
            self.current_node = entered;
            // We have moved to a completely unexplored, unvisited node
            self.new_visit = true;
            let step = Step::Forward {
                left,
                edge,
                entered,
                left_fully_explored,
            };
            self.prev_step = step;
            Some(step)
        }
    }
}
