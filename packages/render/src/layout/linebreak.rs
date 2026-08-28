//! The Knuth & Plass line-breaking algorithm, ported from the implementation
//! react-pdf uses (`@react-pdf/textkit`, itself Bram Stein's BSD-licensed JS).
//!
//! Greedy wrapping is not sufficient for parity. The reference breaker treats
//! inter-word spaces as elastic glue that may shrink, so it will keep a word on
//! a line that overflows the column by a fraction of a point — and then set the
//! line slightly tight. Reproducing where the text breaks means reproducing this
//! algorithm, including its demerit weights and fitness classes.

const INFINITY: f64 = 10_000.0;

const DEMERITS_LINE: f64 = 10.0;
const DEMERITS_FLAGGED: f64 = 100.0;
const DEMERITS_FITNESS: f64 = 3_000.0;

/// react-pdf's glue elasticity: a space may stretch to 3/2 and shrink to 2/3 of
/// its natural width (`{ width: 3, stretch: 6, shrink: 9 }`).
const GLUE_STRETCH_RATIO: f64 = 3.0 / 6.0;
const GLUE_SHRINK_RATIO: f64 = 3.0 / 9.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Box,
    Glue,
    Penalty,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub width: f64,
    pub stretch: f64,
    pub shrink: f64,
    pub penalty: f64,
    pub flagged: f64,
}

impl Node {
    pub fn text_box(width: f64) -> Self {
        Self {
            kind: Kind::Box,
            width,
            stretch: 0.0,
            shrink: 0.0,
            penalty: 0.0,
            flagged: 0.0,
        }
    }

    pub fn glue(width: f64) -> Self {
        Self {
            kind: Kind::Glue,
            width,
            stretch: width * GLUE_STRETCH_RATIO,
            shrink: width * GLUE_SHRINK_RATIO,
            penalty: 0.0,
            flagged: 0.0,
        }
    }

    fn raw_glue(width: f64, stretch: f64, shrink: f64) -> Self {
        Self {
            kind: Kind::Glue,
            width,
            stretch,
            shrink,
            penalty: 0.0,
            flagged: 0.0,
        }
    }

    fn penalty(width: f64, penalty: f64, flagged: f64) -> Self {
        Self {
            kind: Kind::Penalty,
            width,
            stretch: 0.0,
            shrink: 0.0,
            penalty,
            flagged,
        }
    }
}

/// Close a node list with the mandatory final break, as the reference does.
pub fn terminate(nodes: &mut Vec<Node>) {
    nodes.push(Node::raw_glue(0.0, INFINITY, 0.0));
    nodes.push(Node::penalty(0.0, -INFINITY, 1.0));
}

#[derive(Clone, Copy, Default, Debug)]
struct Totals {
    width: f64,
    stretch: f64,
    shrink: f64,
}

#[derive(Clone, Debug)]
struct Breakpoint {
    position: usize,
    demerits: f64,
    line: usize,
    fitness_class: usize,
    totals: Totals,
    previous: Option<usize>,
}

struct Breaker<'a> {
    nodes: &'a [Node],
    line_length: f64,
    tolerance: f64,
    sum: Totals,
    arena: Vec<Breakpoint>,
    active: Vec<usize>,
}

impl<'a> Breaker<'a> {
    fn cost(&self, end: usize, active_totals: &Totals) -> f64 {
        let mut width = self.sum.width - active_totals.width;
        if self.nodes[end].kind == Kind::Penalty {
            width += self.nodes[end].width;
        }

        if width < self.line_length {
            let stretch = self.sum.stretch - active_totals.stretch;
            if stretch > 0.0 {
                (self.line_length - width) / stretch
            } else {
                INFINITY
            }
        } else if width > self.line_length {
            let shrink = self.sum.shrink - active_totals.shrink;
            if shrink > 0.0 {
                (self.line_length - width) / shrink
            } else {
                INFINITY
            }
        } else {
            0.0
        }
    }

    /// Width, stretch and shrink from a breakpoint up to the next box or forced
    /// penalty.
    fn totals_after(&self, from: usize) -> Totals {
        let mut result = self.sum;
        for (index, node) in self.nodes.iter().enumerate().skip(from) {
            match node.kind {
                Kind::Glue => {
                    result.width += node.width;
                    result.stretch += node.stretch;
                    result.shrink += node.shrink;
                }
                Kind::Box => break,
                Kind::Penalty if node.penalty == -INFINITY && index > from => break,
                Kind::Penalty => {}
            }
        }
        result
    }

    fn main_loop(&mut self, index: usize) {
        let node = self.nodes[index].clone();
        let forced = node.kind == Kind::Penalty && node.penalty == -INFINITY;
        let mut cursor = 0usize;

        loop {
            // Best candidate per fitness class: (active arena index, demerits).
            let mut candidates: [Option<(usize, f64)>; 4] = [None; 4];
            let mut current_line;

            // Inner loop: walk active nodes belonging to the current line.
            let at_end = loop {
                if cursor >= self.active.len() {
                    break true;
                }

                let active_index = self.active[cursor];
                let active = self.arena[active_index].clone();
                current_line = active.line + 1;
                let ratio = self.cost(index, &active.totals);

                let deactivate = ratio < -1.0 || forced;
                if deactivate {
                    self.active.remove(cursor);
                }

                if ratio >= -1.0 && ratio <= self.tolerance {
                    let badness = 100.0 * ratio.abs().powi(3);

                    let mut demerits = if node.kind == Kind::Penalty && node.penalty >= 0.0 {
                        (DEMERITS_LINE + badness).powi(2) + node.penalty.powi(2)
                    } else if node.kind == Kind::Penalty && node.penalty != -INFINITY {
                        (DEMERITS_LINE + badness).powi(2) - node.penalty.powi(2)
                    } else {
                        (DEMERITS_LINE + badness).powi(2)
                    };

                    if node.kind == Kind::Penalty
                        && self.nodes[active.position].kind == Kind::Penalty
                    {
                        demerits +=
                            DEMERITS_FLAGGED * node.flagged * self.nodes[active.position].flagged;
                    }

                    let class: usize = if ratio < -0.5 {
                        0
                    } else if ratio <= 0.5 {
                        1
                    } else if ratio <= 1.0 {
                        2
                    } else {
                        3
                    };

                    if class.abs_diff(active.fitness_class) > 1 {
                        demerits += DEMERITS_FITNESS;
                    }
                    demerits += active.demerits;

                    if candidates[class].is_none_or(|(_, best)| demerits < best) {
                        candidates[class] = Some((active_index, demerits));
                    }
                }

                if !deactivate {
                    cursor += 1;
                }

                if cursor >= self.active.len() {
                    break true;
                }
                if self.arena[self.active[cursor]].line >= current_line {
                    break false;
                }
            };

            let totals = self.totals_after(index);
            for (class, candidate) in candidates.iter().enumerate() {
                let Some((active_index, demerits)) = *candidate else {
                    continue;
                };
                let new_index = self.arena.len();
                self.arena.push(Breakpoint {
                    position: index,
                    demerits,
                    line: self.arena[active_index].line + 1,
                    fitness_class: class,
                    totals,
                    previous: Some(active_index),
                });

                if at_end {
                    self.active.push(new_index);
                } else {
                    self.active.insert(cursor, new_index);
                    cursor += 1;
                }
            }

            if at_end || cursor >= self.active.len() {
                break;
            }
        }
    }
}

/// Break `nodes` into lines of `line_length`, returning the node indices where
/// breaks occur (the leading position 0 is dropped, as in the reference).
///
/// Returns `None` when no feasible set of breaks exists at this tolerance.
pub fn break_lines(nodes: &[Node], line_length: f64, tolerance: f64) -> Option<Vec<usize>> {
    let mut breaker = Breaker {
        nodes,
        line_length,
        tolerance,
        sum: Totals::default(),
        arena: vec![Breakpoint {
            position: 0,
            demerits: 0.0,
            line: 0,
            fitness_class: 0,
            totals: Totals::default(),
            previous: None,
        }],
        active: vec![0],
    };

    for index in 0..nodes.len() {
        match nodes[index].kind {
            Kind::Box => breaker.sum.width += nodes[index].width,
            Kind::Glue => {
                if index > 0 && nodes[index - 1].kind == Kind::Box {
                    breaker.main_loop(index);
                }
                breaker.sum.width += nodes[index].width;
                breaker.sum.stretch += nodes[index].stretch;
                breaker.sum.shrink += nodes[index].shrink;
            }
            Kind::Penalty if nodes[index].penalty != INFINITY => breaker.main_loop(index),
            Kind::Penalty => {}
        }
    }

    if breaker.active.is_empty() {
        return None;
    }

    let best = breaker.active.iter().copied().min_by(|a, b| {
        breaker.arena[*a]
            .demerits
            .total_cmp(&breaker.arena[*b].demerits)
    })?;

    let mut positions = Vec::new();
    let mut node = Some(best);
    while let Some(index) = node {
        positions.push(breaker.arena[index].position);
        node = breaker.arena[index].previous;
    }
    positions.reverse();

    // Drop the paragraph-start breakpoint.
    Some(positions.into_iter().skip(1).collect())
}
