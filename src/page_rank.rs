use crate::PageGraph;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
pub struct PageRank {
	rank: Vec<f64>,
}
impl PageRank {
	pub fn build(page_graph: &PageGraph, num_pages: usize) -> Self {
		Self {
			rank: Self::pagerank_impl(page_graph, num_pages, 25, 0.85),
		}
	}

	#[inline]
	pub fn get_rank(&self) -> &[f64] {
		&self.rank
	}

	fn pagerank_impl(
		page_graph: &PageGraph,
		n: usize,
		iterations: usize,
		damping: f64,
	) -> Vec<f64> {
		if n == 0 {
			return Vec::new();
		}

		let n_f64 = n as f64;
		let base = (1.0 - damping) / n_f64;

		// Current and next rank vectors
		let mut rank = vec![1.0 / n_f64; n];
		let mut next = vec![0.0; n];

		// Precompute out-degree reciprocals
		let mut inv_out_degree = vec![0.0; n];
		for (i, inv_out_degree_value) in inv_out_degree.iter_mut().enumerate() {
			let deg = (page_graph.forward_offsets[i + 1] - page_graph.forward_offsets[i]) as f64;
			if deg > 0.0 {
				*inv_out_degree_value = 1.0 / deg;
			}
		}

		for _ in 0..iterations {
			// Reset next with teleport/base value
			next.fill(base);

			// Sum of dangling nodes (no outgoing edges)
			let mut dangling_sum = 0.0;

			// Distribute rank
			for i in 0..n {
				let start = page_graph.forward_offsets[i] as usize;
				let end = page_graph.forward_offsets[i + 1] as usize;

				if start == end {
					// Dangling node
					dangling_sum += rank[i];
					continue;
				}

				let contribution = rank[i] * inv_out_degree[i] * damping;

				// Sequential scan over adjacency
				for &dst in &page_graph.forward_edges[start..end] {
					next[dst as usize] += contribution;
				}
			}

			// Distribute dangling mass uniformly
			let dangling_contribution = damping * dangling_sum / n_f64;
			for v in &mut next {
				*v += dangling_contribution;
			}

			// Swap buffers
			std::mem::swap(&mut rank, &mut next);
		}

		rank
	}
}
