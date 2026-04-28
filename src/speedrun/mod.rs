mod dfs;
pub use dfs::speedrun_any_percent_dfs;

mod bfs;
pub use bfs::speedrun_any_percent_bfs;

mod bidirectional;
pub use bidirectional::speedrun_any_percent_bidirectional_bfs;

mod shortest_path;
pub use shortest_path::speedrun_shortest_path;

use crate::{ArchivedPageGraph, DensePageId};

pub type SpeedrunAlgorithm = fn(
	start_dense_page_id: DensePageId,
	end_dense_page_id: DensePageId,
	page_graph: &ArchivedPageGraph,
) -> Option<Vec<DensePageId>>;
