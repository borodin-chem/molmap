// SPDX-FileCopyrightText: 2026 Matthew Milner <matterhorn103@proton.me>
//
// SPDX-License-Identifier: MPL-2.0
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

use crate::{
    Element,
    categories::*,
    entities::*,
    error::*,
    graph::{MolGraph, keys::Keyed},
};

/// The nature of a transition from one graph node to another.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NextStepContext {
    /// A step along a new branch from a new node in a completely unexplored network.
    Initial,
    /// A step from the current node, going deeper along the current branch.
    Continuation,
    /// A step along a new branch after backtracking to a previously visited node.
    Backtracked,
    /// There are no remaining steps to be made.
    Finished,
}

/// The nature of the graph node that is the destination of a traversal step.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DestStatus {
    /// The node has not been previously visited, and has undiscovered edges.
    Unexplored,
    /// The node is a hydrogen atom and has no other edges out of it other than the one
    /// which was just followed to it.
    HydrogenDeadEnd,
    /// The node has no other edges out of it other than the one which was just
    /// followed to it, and the node is _not_ a hydrogen atom.
    DeadEnd,
    /// The node has been previously visited, meaning that the step has closed a cycle.
    /// The node may or may not have remaining undiscovered edges.
    Visited,
}

#[derive(Copy, Clone, Debug)]
pub struct Step {
    context: NextStepContext,
    status: DestStatus,
    origin: AnyBondable,
    edge: Bond,
    dest: AnyBondable,
}

#[derive(Clone, Debug)]
struct StackItem {
    node: AnyBondable,
    hydrogen_queue: Vec<QueueItem>,
    queue: Vec<QueueItem>,
}

#[derive(Clone, Debug)]
struct QueueItem {
    edge: Bond,
    dest: AnyBondable,
}

/// An iterator over traversal steps.
#[derive(Debug)]
pub struct DepthFirstSearch<'m> {
    graph: &'m MolGraph,
    current_node: AnyBondable,
    current_context: NextStepContext,
    visited: HashSet<AnyBondable>,
    discovered: HashSet<Bond>,
    stack: Vec<StackItem>,
    hydrogen_queue: Vec<QueueItem>,
    queue: Vec<QueueItem>,
}

impl<'m> DepthFirstSearch<'m> {
    /// Initializes a traversal of the graph from an arbitrary atom or pseudoatom.
    ///
    /// # Errors
    ///
    /// Fails if the graph does not contain any atoms or pseudoatoms at all.
    pub fn new(graph: &'m MolGraph) -> MolMapResult<Self> {
        let start: AnyAtomlike = {
            if let Some(key) = graph.atoms.keys().next() {
                Atom::from_key(key).as_atomlike()
            } else if let Some(key) = graph.pseudoatoms.keys().next() {
                Pseudoatom::from_key(key).as_atomlike()
            } else {
                // Early return of empty iterator over apparently empty graph
                return Err(MolMapError::EmptyMap);
            }
        };
        DepthFirstSearch::new_with_start(graph, start)
    }

    /// Initializes a traversal of the graph from the indicated atom or pseudoatom.
    ///
    /// # Errors
    ///
    /// Fails if the graph does not contain any atoms or pseudoatoms at all.
    pub fn new_with_start(graph: &'m MolGraph, start: AnyAtomlike) -> MolMapResult<Self> {
        let atomlike_count = graph.atoms.len() + graph.pseudoatoms.len();
        if atomlike_count == 0 {
            Err(MolMapError::EmptyMap)
        } else {
            Ok(Self {
                graph,
                current_node: start.as_bondable(),
                current_context: NextStepContext::Initial,
                visited: HashSet::with_capacity(atomlike_count),
                discovered: HashSet::with_capacity(graph.bonds.len()),
                stack: Vec::with_capacity(atomlike_count / 2), // Assume at least half of atoms are terminal
                hydrogen_queue: Vec::with_capacity(6),
                queue: Vec::with_capacity(6),
            })
        }
    }
}

impl<'m> Iterator for DepthFirstSearch<'m> {
    type Item = Step;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_context == NextStepContext::Finished {
            return None;
        };
        self.visited.insert(self.current_node);
        if self.queue.is_empty() && self.hydrogen_queue.is_empty() {
            // Populate queues with all eligible bonds
            let outward_bonds = match self.current_node.resolve() {
                ResolvedBondable::Atom(atom) => &self.graph.data(atom).unwrap().bonds,
                ResolvedBondable::Pseudoatom(pseudoatom) => {
                    &self.graph.data(pseudoatom).unwrap().bonds
                }
            };
            for &bond in outward_bonds {
                // Skip bond if it has already been discovered (i.e. we came in to the
                // node that way)
                if self.discovered.contains(&bond) {
                    continue;
                }
                let bond_data = self.graph.data(bond).unwrap();
                // Skip any weak bonds, they don't count as edges
                if !bond_data.bond_type.is_strong() {
                    continue;
                }
                // OK, this bond is a valid candidate for the next step, we want to add
                // it to the queue
                // First need to work out what the other end of the bond is
                let dest = if bond_data.start != self.current_node {
                    bond_data.start
                } else {
                    bond_data.end
                };
                // Split hydrogen and non-hydrogen destinations into two separate queues
                match dest.resolve() {
                    ResolvedBondable::Atom(atom) => {
                        if self.graph.data(atom).unwrap().element == Element::H {
                            self.hydrogen_queue.push(QueueItem { edge: bond, dest })
                        } else {
                            self.queue.push(QueueItem { edge: bond, dest })
                        }
                    }
                    ResolvedBondable::Pseudoatom(pseudoatom) => {
                        self.queue.push(QueueItem { edge: bond, dest })
                    }
                }
            }
        }
        // There should now definitely be some items in at least one of the queues!
        // Select the next edge to discover, getting adjacent hydrogen atoms out of
        // the way first
        let (next, dest_is_h) = if !self.hydrogen_queue.is_empty() {
            (
                self.hydrogen_queue
                    .pop()
                    .expect("Just checked that queue isn't empty"),
                true,
            )
        } else {
            (
                self.queue
                    .pop()
                    .expect("At least one of the queues should definitely be populated"),
                false,
            )
        };
        let dest_is_visited = self.visited.contains(&next.dest);
        let n_dest_bonds = match next.dest.resolve() {
            ResolvedBondable::Atom(atom) => self.graph.data(atom).unwrap().bonds.len(),
            ResolvedBondable::Pseudoatom(pseudoatom) => {
                self.graph.data(pseudoatom).unwrap().bonds.len()
            }
        };
        let dest_is_deadend = (n_dest_bonds <= 1);
        let status = if dest_is_visited {
            DestStatus::Visited
        } else if dest_is_deadend {
            if dest_is_h {
                DestStatus::HydrogenDeadEnd
            } else {
                DestStatus::DeadEnd
            }
        } else {
            DestStatus::Unexplored
        };
        let step = Step {
            context: self.current_context,
            status,
            origin: self.current_node,
            edge: next.edge,
            dest: next.dest,
        };
        // Update state now that we've assembled the return value
        self.discovered.insert(next.edge);
        if dest_is_deadend {
            // If we arrived at a dead-end, we just go again, but technically we have moved
            // and then back-tracked
            //   current_node -> stays the same
            //   current_context -> back-tracked
            //   visited -> briefly visited dead-end gets added
            //   discovered -> discovered edge was just added
            //   stack -> no change
            //   hydrogen_queue -> retain for next go
            //   queue -> retain for next go
            self.current_context = NextStepContext::Backtracked;
            // Normally we "visit" a node at the start of next() when it's the current
            // node, but we didn't bother actually moving to it as it's a dead-end, but
            // have to make sure it's recorded as visited
            self.visited.insert(next.dest);
            // Stack and queues don't change
            todo!()
            // It is possible that the queue for the current node is now exhausted, in
            // which case we'll have to back-track further.
            // It might even be that there are no more connected nodes that we can reach,
            // in which case we need to jump to a new network.
            // It might even be the case that we have iterated over everything.
        } else if dest_is_visited {
            // Closed a cycle, bit complicated
            //   current_node -> ?
            //   current_context -> ?
            //   visited -> no change
            //   discovered -> discovered edge was just added
            //   stack -> ?
            //   hydrogen_queue -> ?
            //   queue -> ?
            todo!()
        } else {
            // We have properly moved to a new node and need to update accordingly
            //   current_node -> update
            //   current_context -> continuation
            //   visited -> no change (new node will be added next go)
            //   discovered -> discovered edge was just added
            //   stack -> put node we just left onto stack
            //   hydrogen_queue -> drain
            //   queue -> drain
            // The node we just left gets added to the stack together with its cached queues -
            // unless the queues are empty, in which case we'll never have to come back to it
            if !self.queue.is_empty() || !self.hydrogen_queue.is_empty() {
                self.stack.push(StackItem {
                    node: self.current_node,
                    hydrogen_queue: self.hydrogen_queue.drain(..).collect(),
                    queue: self.queue.drain(..).collect(),
                });
            }
            self.current_node = next.dest;
            self.current_context = NextStepContext::Continuation;
        }
        Some(step)
    }
}
