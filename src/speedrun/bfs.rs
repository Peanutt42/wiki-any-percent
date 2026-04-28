use crate::{ArchivedPageGraph, DensePageId};
use std::collections::VecDeque;

pub fn speedrun_any_percent_bfs(
	start: DensePageId,
	end: DensePageId,
	graph: &ArchivedPageGraph,
) -> Option<Vec<DensePageId>> {
	let n = graph.forward_offsets.len() - 1;

	let mut visited = vec![false; n];
	let mut parent: Vec<Option<DensePageId>> = vec![None; n];
	let mut queue = VecDeque::new();

	visited[start as usize] = true;
	queue.push_back(start);

	while let Some(node) = queue.pop_front() {
		for neighbor in graph.iter_forward_linked_pages(node) {
			let neighbor = neighbor.to_native();
			let idx = neighbor as usize;

			if visited[idx] {
				continue;
			}

			visited[idx] = true;
			parent[idx] = Some(node);

			if neighbor == end {
				// reconstruct path
				let mut path = Vec::new();
				let mut current = end;

				while let Some(p) = parent[current as usize] {
					path.push(current);
					current = p;
				}

				path.push(start);
				path.reverse();
				return Some(path);
			}

			queue.push_back(neighbor);
		}
	}

	None
}
