/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
use anyhow::Result;
use lender::Lender;
use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::time::{Duration, SystemTime};
use webgraph::algo::GeometricCentralities;
use webgraph::graphs::random::ErdosRenyi;
use webgraph::labels::Left;
use webgraph::prelude::VecGraph;
use webgraph::traits::{RandomAccessGraph, SequentialLabeling};

pub fn main() -> Result<()> {
    env_logger::init();
    // Call the main function of the CLI with cli args
    //cli_main(std::env::args_os())

    //let path = "/home/matteo/Documents/tesi/example_graphs/uk-2007-05@100000/".to_owned();
    //let graph = BvGraph::with_basename(path + "uk-2007-05@100000").load()?;

    let nodes = 100_000;
    let edge_prob = 20f64 / nodes as f64;
    let iterations = 3;
    benchmark_geom_on_random_graph_rayon(nodes, edge_prob, iterations, 1);
    benchmark_geom_on_random_graph_rayon(nodes, edge_prob, iterations, 10);
    //benchmark_geom_on_random_graph_atomic_counter_out_channel(nodes, edge_prob, iterations);
    //benchmark_geom_on_random_graph_2_channels(nodes, edge_prob, iterations);

    println!("Done");
    Ok(())
}

fn benchmark_geom_on_random_graph_rayon(
    nodes: usize,
    edge_prob: f64,
    iterations: usize,
    chunk_size: usize,
) {
    eprintln!("-------------------------------------");
    eprintln!("Computing geometric centralities rayon...");
    let mut tot_time = Duration::new(0, 0);
    for i in 0..iterations {
        let graph = gen_random_graph(nodes, edge_prob, i as u64);
        let mut geom = GeometricCentralities::new(&graph, 0, true);
        let time_before = SystemTime::now();
        geom.compute_with_par_iter(chunk_size);
        let delta_time = time_before.elapsed().expect("Error in delta_time");
        tot_time += delta_time;
        eprintln!("Done computing geom {i}")
    }
    eprintln!(
        "avg rayon chunks: {chunk_size} {}ms",
        tot_time.as_millis() as f64 / iterations as f64
    );
    eprintln!("-------------------------------------");
    //println!("{:?}", geom.closeness)
}

fn benchmark_geom_on_random_graph_atomic_counter_out_channel(
    nodes: usize,
    edge_prob: f64,
    iterations: usize,
) {
    eprintln!("-------------------------------------");
    eprintln!("Computing geometric centralities atomic out channel...");
    let mut tot_time = Duration::new(0, 0);
    for i in 0..iterations {
        let graph = gen_random_graph(nodes, edge_prob, i as u64);
        let mut geom = GeometricCentralities::new(&graph, 0, true);
        let time_before = SystemTime::now();
        geom.compute_with_atomic_counter_out_channel();
        let delta_time = time_before.elapsed().expect("Error in delta_time");
        tot_time += delta_time;
        eprintln!("Done computing geom {i}")
    }
    eprintln!(
        "avg atomic out channels {}ms",
        tot_time.as_millis() as f64 / iterations as f64
    );
    eprintln!("-------------------------------------");
    //println!("{:?}", geom.closeness)
}

fn benchmark_geom_on_random_graph_2_channels(nodes: usize, edge_prob: f64, iterations: usize) {
    eprintln!("-------------------------------------");
    eprintln!("Computing geometric centralities 2 channels...");
    let mut tot_time = Duration::new(0, 0);
    for i in 0..iterations {
        let graph = gen_random_graph(nodes, edge_prob, i as u64);
        let mut geom = GeometricCentralities::new(&graph, 0, true);
        let time_before = SystemTime::now();
        geom.compute_with_2_channels();
        let delta_time = time_before.elapsed().expect("Error in delta_time");
        tot_time += delta_time;
        eprintln!("Done computing geom {i}")
    }
    eprintln!(
        "avg atomic out channels thread pool {}ms",
        tot_time.as_millis() as f64 / iterations as f64
    );
    eprintln!("-------------------------------------");
    //println!("{:?}", geom.closeness)
}

fn gen_random_graph(nodes: usize, edge_prob: f64, seed: u64) -> impl RandomAccessGraph {
    let rand = ErdosRenyi::new(nodes, edge_prob, seed);
    let vg = VecGraph::from_lender(rand.iter());
    println!("Generated random graph!");
    Left(vg)
}


fn write_results(geom: &GeometricCentralities<impl RandomAccessGraph>) {
    let mut file = fs::File::create("/home/matteo/Documents/tesi/data/rust/rust_closeness")
        .expect("Can't create file closeness");
    let text: String = geom.closeness.iter().map(|n| format!("{}\n", n)).collect();
    file.write_all(&text.into_bytes())
        .expect("Can't write closeness to file");

    let mut file = fs::File::create("/home/matteo/Documents/tesi/data/rust/rust_harmonic")
        .expect("Can't create file harmonic");
    let text: String = geom.harmonic.iter().map(|n| format!("{}\n", n)).collect();
    file.write_all(&text.into_bytes())
        .expect("Can't write harmonic to file");

    let mut file = fs::File::create("/home/matteo/Documents/tesi/data/rust/rust_lin")
        .expect("Can't create lin");
    let text: String = geom.lin.iter().map(|n| format!("{}\n", n)).collect();
    file.write_all(&text.into_bytes())
        .expect("Can't write lin to file");

    let mut file = fs::File::create("/home/matteo/Documents/tesi/data/rust/rust_exponential")
        .expect("Can't create exponential");
    let text: String = geom
        .exponential
        .iter()
        .map(|n| format!("{}\n", n))
        .collect();
    file.write_all(&text.into_bytes())
        .expect("Can't write exponential to file");

    let mut file = fs::File::create("/home/matteo/Documents/tesi/data/rust/rust_reachable")
        .expect("Can't create reachable");
    let text: String = geom.reachable.iter().map(|n| format!("{}\n", n)).collect();
    file.write_all(&text.into_bytes())
        .expect("Can't write reachable to file");
}


fn print_graph(g: &impl RandomAccessGraph) {
    println!("Printing graph...");
    for node in 0..g.num_nodes() {
        println!(
            "{node}: {:?}",
            g.successors(node).into_iter().collect::<Vec<_>>()
        );
    }
}

fn transpose_arc_list(
    arcs: impl IntoIterator<Item=(usize, usize)>,
) -> impl IntoIterator<Item=(usize, usize)> {
    arcs.into_iter().map(|(a, b)| (b, a))
}

fn bfs(vg: &impl RandomAccessGraph) -> Vec<i32> {
    let mut queue = VecDeque::new();
    let mut nodes_iter = vg.iter();
    let mut distances = vec![-1; vg.num_nodes()];
    let first_node = nodes_iter.next().unwrap().0;
    distances[first_node] = 0;
    queue.push_back(first_node);
    while let Some(cur_node) = queue.pop_front() {
        let d = distances[cur_node] + 1;
        let successors = vg.successors(cur_node);
        for next in successors {
            if distances[next] == -1 {
                distances[next] = d;
                queue.push_back(next);
            }
        }
    }
    distances
}
