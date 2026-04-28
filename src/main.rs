use clap::{Parser, Subcommand};
use memmap::Mmap;
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf, time::Instant};
use wiki_any_percent::{
	ArchivedPageGraph, ArchivedPageNames, DensePageId, DensePageIdMapBuilder, PageEdge, PageGraph,
	PageNameToDensePageIdEntry, PageNames, SpeedrunAlgorithm,
	convert_to_wikipedia_page_name_convention, speedrun_any_percent_bfs,
	speedrun_any_percent_bidirectional_bfs, speedrun_any_percent_dfs, speedrun_shortest_path,
	sql_dump_extraction::{
		parse_linktarget_sql_dump, parse_page_sql_dump, parse_pagelink_sql_dump,
	},
};

#[derive(Parser)]
struct Cli {
	#[command(subcommand)]
	action: CliAction,
}

#[derive(Subcommand)]
enum CliAction {
	Extract {
		page_sql_dump_file: PathBuf,
		pagelinks_sql_dump_file: PathBuf,
		linktarget_sql_dump_file: PathBuf,
		out_page_names_file: PathBuf,
		out_page_graph_file: PathBuf,
	},
	BrowseLinkedPages {
		page_names_file: PathBuf,
		page_graph_file: PathBuf,
	},
	Speedrun {
		mode: SpeedrunMode,
		page_names_file: PathBuf,
		page_graph_file: PathBuf,
		start_page_name: String,
		end_page_name: String,
	},
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SpeedrunMode {
	/// as fast as possible, uses DFS which is much faster than BFS here, but results in quite bad
	/// and long paths, though they are valid
	AnyPercentDFS,
	/// as fast as possible (uses BFS)
	AnyPercent,
	AnyPercentBidirectional,
	/// least amount of jumps from start to end
	ShortestPath,
}
impl SpeedrunMode {
	fn get_algo(self) -> SpeedrunAlgorithm {
		match self {
			Self::AnyPercentDFS => speedrun_any_percent_dfs,
			Self::AnyPercent => speedrun_any_percent_bfs,
			Self::AnyPercentBidirectional => speedrun_any_percent_bidirectional_bfs,
			Self::ShortestPath => speedrun_shortest_path,
		}
	}
}

fn browse_linked_pages(page_names_filepath: PathBuf, page_graph_filepath: PathBuf) {
	let load_page_names_start = Instant::now();
	let page_names_file = File::open(&page_names_filepath).unwrap();
	let page_names_bytes = unsafe { Mmap::map(&page_names_file) }.unwrap();
	let page_names =
		rkyv::access::<ArchivedPageNames, rkyv::rancor::Error>(&page_names_bytes).unwrap();
	println!(
		"finished reading {}, took {} seconds",
		page_names_filepath.display(),
		load_page_names_start.elapsed().as_secs_f32()
	);

	let load_page_graph_start = Instant::now();
	let page_graph_file = File::open(&page_graph_filepath).unwrap();
	let page_graph_bytes = unsafe { Mmap::map(&page_graph_file) }.unwrap();
	let page_graph =
		rkyv::access::<ArchivedPageGraph, rkyv::rancor::Error>(&page_graph_bytes).unwrap();
	println!(
		"finished reading {}, took {} seconds",
		page_graph_filepath.display(),
		load_page_graph_start.elapsed().as_secs_f32()
	);

	let mut input = None;

	loop {
		if input.is_none() {
			let mut new_input = String::new();
			print!("> ");
			std::io::stdout().flush().unwrap();
			std::io::stdin().read_line(&mut new_input).unwrap();
			input = Some(
				new_input
					.strip_suffix('\n')
					.unwrap_or(new_input.as_str())
					.to_string(),
			);
		}

		if input.is_none() {
			continue;
		}

		let start = Instant::now();
		match page_names.lookup_name(input.as_ref().unwrap().as_str()) {
			Some(dense_page_id) => {
				let linked_page_names = page_graph
					.iter_forward_linked_pages(dense_page_id)
					.map(|linked_dense_page_id| {
						page_names
							.lookup_dense_page_id(linked_dense_page_id.to_native())
							.map(str::to_string)
							.unwrap_or(format!(
								"no page name could be found for dense page id {linked_dense_page_id}"
							))
					})
					.collect::<Vec<_>>();

				println!(
					"took {} milliseconds",
					start.elapsed().as_secs_f32() * 1000.0
				);

				match inquire::Select::new("", linked_page_names)
					.with_page_size(20)
					.raw_prompt_skippable()
				{
					Ok(Some(selection)) => {
						input = Some(selection.value);
					}
					Ok(None) => {
						input = None;
					}
					Err(e) => {
						input = None;
						eprintln!("{e}");
					}
				}
			}
			None => println!("Nope..."),
		}
	}
}

fn extract(
	page_sql_dump_file: PathBuf,
	pagelinks_sql_dump_file: PathBuf,
	linktarget_sql_dump_file: PathBuf,
	out_page_names_file: PathBuf,
	out_page_graph_file: PathBuf,
) {
	println!("loading page sql dump...");
	let mut dense_page_id_map_builder = DensePageIdMapBuilder::new();
	let mut page_name_to_dense_id_entries = Vec::new();
	parse_page_sql_dump(&page_sql_dump_file, |page| {
		let dense_page_id = dense_page_id_map_builder.push(page.page_id);
		let name = String::from_utf8(page.page_title).unwrap();
		page_name_to_dense_id_entries.push(PageNameToDensePageIdEntry {
			name,
			dense_page_id,
		});
		if page_name_to_dense_id_entries.len() % 1_000_000 == 0 {
			println!("{}", page_name_to_dense_id_entries.len());
		}
	});

	let dense_page_id_map = dense_page_id_map_builder.build();
	let num_pages = dense_page_id_map.0.len();
	println!("total of {num_pages} pages",);

	println!("buidling page names... (sorting...)");
	let page_names = PageNames::build(page_name_to_dense_id_entries);

	println!("saving page names...");
	page_names.save(&out_page_names_file);

	println!("loading linktarget sql dump...");
	let mut linktarget_to_dense_page_id: HashMap<u32, DensePageId> = HashMap::new();
	parse_linktarget_sql_dump(linktarget_sql_dump_file, |linktarget| {
		let page_name = String::from_utf8(linktarget.lt_title).unwrap();
		if let Some(dense_page_id) = page_names.lookup_name(&page_name) {
			linktarget_to_dense_page_id.insert(linktarget.lt_id as u32, dense_page_id);
			if linktarget_to_dense_page_id.len().is_multiple_of(1_000_000) {
				println!("{}", linktarget_to_dense_page_id.len());
			}
		}
	});
	println!("total of {} linktargets", linktarget_to_dense_page_id.len());

	println!("loading pagelinks sql dump...");
	// from -> to
	let mut page_edges: Vec<PageEdge> = Vec::new();
	parse_pagelink_sql_dump(pagelinks_sql_dump_file, |pagelink| {
		if let Some(from_dense_page_id) = dense_page_id_map.get_dense_id(pagelink.pl_from)
			&& let Some(to_dense_page_id) =
				linktarget_to_dense_page_id.get(&(pagelink.pl_target_id as u32))
		{
			page_edges.push(PageEdge::new(*from_dense_page_id, *to_dense_page_id));

			if page_edges.len().is_multiple_of(10_000_000) {
				println!("{}", page_edges.len());
			}
		}
	});
	println!("building page graph...");
	let page_graph = PageGraph::build(page_edges.iter(), page_edges.iter(), num_pages);

	println!("saving page graph...");
	page_graph.save(out_page_graph_file);
}

fn speedrun(
	mode: SpeedrunMode,
	page_names_filepath: PathBuf,
	page_graph_filepath: PathBuf,
	start_page_name: String,
	end_page_name: String,
) {
	let load_page_names_start = Instant::now();
	let page_names_file = File::open(&page_names_filepath).unwrap();
	let page_names_bytes = unsafe { Mmap::map(&page_names_file) }.unwrap();
	let page_names =
		rkyv::access::<ArchivedPageNames, rkyv::rancor::Error>(&page_names_bytes).unwrap();
	println!(
		"finished reading {}, took {} seconds",
		page_names_filepath.display(),
		load_page_names_start.elapsed().as_secs_f32()
	);

	let load_page_graph_start = Instant::now();
	let page_graph_file = File::open(&page_graph_filepath).unwrap();
	let page_graph_bytes = unsafe { Mmap::map(&page_graph_file) }.unwrap();
	let page_graph =
		rkyv::access::<ArchivedPageGraph, rkyv::rancor::Error>(&page_graph_bytes).unwrap();
	println!(
		"finished reading {}, took {} seconds",
		page_graph_filepath.display(),
		load_page_graph_start.elapsed().as_secs_f32()
	);

	let page_name_lookup_start = Instant::now();
	let start_page_name = convert_to_wikipedia_page_name_convention(&start_page_name);
	let end_page_name = convert_to_wikipedia_page_name_convention(&end_page_name);

	let start_dense_page_id = match page_names.lookup_name(&start_page_name) {
		Some(dense_page_id) => dense_page_id,
		None => {
			panic!("could not find page with name: {start_page_name}!");
		}
	};
	let end_dense_page_id = match page_names.lookup_name(&end_page_name) {
		Some(dense_page_id) => dense_page_id,
		None => {
			panic!("could not find page with name: {end_page_name}!");
		}
	};
	println!(
		"name convertion and lookup took: {:.3} milliseconds",
		page_name_lookup_start.elapsed().as_secs_f32() * 1000.0
	);

	let speedrun_start = Instant::now();
	let result = mode.get_algo()(start_dense_page_id, end_dense_page_id, page_graph);

	if let Some(trail) = result {
		println!(
			"Path found, took {:.2} milliseconds",
			speedrun_start.elapsed().as_secs_f32() * 1000.0
		);
		let trail_formatted = trail
			.into_iter()
			.map(|dense_page_id| {
				let page_name = page_names
					.lookup_dense_page_id(dense_page_id)
					.unwrap_or("<page name not found>");
				format!("{page_name} ({dense_page_id})")
			})
			.collect::<Vec<String>>()
			.join("\n -> ");

		println!("{trail_formatted}");
	} else {
		panic!(
			"no possible path was found! (took {:.2} milliseconds)",
			speedrun_start.elapsed().as_secs_f32() * 1000.0
		);
	}
}

fn main() {
	let cli = Cli::parse();

	match cli.action {
		CliAction::Extract {
			page_sql_dump_file,
			pagelinks_sql_dump_file,
			linktarget_sql_dump_file,
			out_page_names_file,
			out_page_graph_file,
		} => {
			extract(
				page_sql_dump_file,
				pagelinks_sql_dump_file,
				linktarget_sql_dump_file,
				out_page_names_file,
				out_page_graph_file,
			);
		}
		CliAction::BrowseLinkedPages {
			page_names_file,
			page_graph_file,
		} => {
			browse_linked_pages(page_names_file, page_graph_file);
		}
		CliAction::Speedrun {
			mode,
			page_names_file,
			page_graph_file,
			start_page_name,
			end_page_name,
		} => {
			speedrun(
				mode,
				page_names_file,
				page_graph_file,
				start_page_name,
				end_page_name,
			);
		}
	}
}
