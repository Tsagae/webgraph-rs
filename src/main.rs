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
use webgraph::algo::GeometricCentralities;
use webgraph::prelude::BvGraph;
use webgraph::traits::{RandomAccessGraph, SequentialLabeling};

pub fn main() -> Result<()> {
    env_logger::init();
    // Call the main function of the CLI with cli args
    //cli_main(std::env::args_os())

    let path = "/home/matteo/Documents/tesi/example_graphs/enron/".to_owned();
    let graph = BvGraph::with_basename(path + "enron").load()?;
    let mut geom = GeometricCentralities::new(&graph, 0, true);
    geom.compute_with_atomic_counter_out_channel();
    let mut geom = GeometricCentralities::new(&graph, 0, true);
    geom.compute_with_2_channels();
    let mut geom = GeometricCentralities::new(&graph, 0, true);
    geom.compute_with_par_iter(1);
    let mut geom = GeometricCentralities::new(&graph, 0, true);
    geom.compute_with_par_iter(10);
    let mut geom = GeometricCentralities::new(&graph, 0, true);
    geom.compute_with_par_iter(50);

    println!("Done");
    Ok(())
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
