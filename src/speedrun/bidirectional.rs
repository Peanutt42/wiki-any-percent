use crate::{ArchivedPageGraph, DensePageId};
use std::collections::VecDeque;

pub fn speedrun_any_percent_bidirectional_bfs(
	start: DensePageId,
	end: DensePageId,
	graph: &ArchivedPageGraph,
) -> Option<Vec<DensePageId>> {
	let n = graph.forward_offsets.len() - 1;

	let mut visited_f = vec![false; n];
	let mut visited_b = vec![false; n];

	let mut parent_f = vec![None; n];
	let mut parent_b = vec![None; n];

	let mut queue_f = VecDeque::new();
	let mut queue_b = VecDeque::new();

	visited_f[start as usize] = true;
	visited_b[end as usize] = true;

	queue_f.push_back(start);
	queue_b.push_back(end);

	loop {
		let expand_forward = queue_f.len() <= queue_b.len();

		if expand_forward && queue_f.is_empty() {
			break;
		}
		if !expand_forward && queue_b.is_empty() {
			break;
		}

		// Expand smaller frontier (important optimization)
		let meet = if expand_forward {
			let node = queue_f.pop_front()?;
			expand(
				node,
				&mut queue_f,
				&mut visited_f,
				&mut parent_f,
				&visited_b,
				graph.iter_forward_linked_pages(node),
			)
		} else {
			let node = queue_b.pop_front()?;
			expand(
				node,
				&mut queue_b,
				&mut visited_b,
				&mut parent_b,
				&visited_f,
				graph.iter_backward_linked_pages(node),
			)
		};

		if let Some(meet_node) = meet {
			return Some(reconstruct_path(meet_node, &parent_f, &parent_b, start));
		}
	}

	None
}
fn expand(
	node: DensePageId,
	queue: &mut VecDeque<DensePageId>,
	visited: &mut [bool],
	parent: &mut [Option<DensePageId>],
	other_visited: &[bool],
	iterator: std::slice::Iter<'_, rkyv::rend::u32_le>,
) -> Option<DensePageId> {
	for neighbor in iterator {
		let neighbor = neighbor.to_native();
		let idx = neighbor as usize;

		if visited[idx] {
			continue;
		}

		visited[idx] = true;
		parent[idx] = Some(node);

		if other_visited[idx] {
			return Some(neighbor); // meeting point
		}

		queue.push_back(neighbor);
	}

	None
}
fn reconstruct_path(
	meet: DensePageId,
	parent_f: &[Option<DensePageId>],
	parent_b: &[Option<DensePageId>],
	start: DensePageId,
) -> Vec<DensePageId> {
	let mut path = Vec::new();

	// forward part
	let mut cur = meet;
	while let Some(p) = parent_f[cur as usize] {
		path.push(cur);
		cur = p;
	}
	path.push(start);
	path.reverse();

	// backward part
	let mut cur = meet;
	while let Some(p) = parent_b[cur as usize] {
		cur = p;
		path.push(cur);
	}

	path
}
