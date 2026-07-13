#!/usr/bin/env python3
"""Run independent GerryChain ReCom traces using BISECT graph inputs."""

from __future__ import annotations

import argparse
import hashlib
import json
import pickle
import random
from functools import partial
from pathlib import Path

import gerrychain
import networkx as nx
import numpy as np
from gerrychain import Graph, MarkovChain, Partition
from gerrychain.accept import always_accept
from gerrychain.constraints import within_percent_of_ideal_population
from gerrychain.proposals import recom
from gerrychain.updaters import Tally, cut_edges


def chain_seed(base_seed: int, index: int) -> int:
    digest = hashlib.sha256(f"GERRYCHAIN_REAL_{index}_{base_seed}".encode()).digest()
    return int.from_bytes(digest[:8], "little")


def load_assignment(path: Path, n: int) -> dict[int, int]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    assignment = {int(index): int(district) for index, district in raw.items()}
    if set(assignment) != set(range(n)):
        raise ValueError("assignment must cover graph nodes 0..n-1")
    return assignment


def democratic_seats(partition: Partition, year: str, k: int) -> int:
    dem = partition[f"dem_{year}"]
    rep = partition[f"rep_{year}"]
    return sum(dem[district] > rep[district] for district in range(1, k + 1))


def max_deviation(partition: Partition, ideal: float, k: int) -> float:
    population = partition["population"]
    return max(abs(population[district] - ideal) / ideal for district in range(1, k + 1))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--state", required=True)
    parser.add_argument("--adjacency", type=Path, required=True)
    parser.add_argument("--assignments", type=Path, required=True)
    parser.add_argument("--elections", type=Path, required=True)
    parser.add_argument("--steps", type=int, required=True)
    parser.add_argument("--chains", type=int, required=True)
    parser.add_argument("--tolerance", type=float, required=True)
    parser.add_argument("--base-seed", type=int, required=True)
    parser.add_argument("--snapshot-stride", type=int, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    payload = pickle.loads(args.adjacency.read_bytes())
    adjacency = payload["adjacency"]
    population = [int(value) for value in payload["vertex_weights"]]
    elections = json.loads(args.elections.read_text(encoding="utf-8"))
    n = len(adjacency)
    assignment = load_assignment(args.assignments, n)
    k = max(assignment.values())
    if elections["state"] != args.state.upper() or elections["geoid_count"] != n:
        raise ValueError("election input does not match state graph")

    nx_graph = nx.Graph()
    for node in range(n):
        nx_graph.add_node(
            node,
            population=population[node],
            dem_2016=float(elections["democratic_2016"][node]),
            rep_2016=float(elections["republican_2016"][node]),
            dem_2020=float(elections["democratic_2020"][node]),
            rep_2020=float(elections["republican_2020"][node]),
        )
        for neighbor in adjacency[node]:
            if node < int(neighbor):
                nx_graph.add_edge(node, int(neighbor))
    graph = Graph.from_networkx(nx_graph)
    updaters = {
        "population": Tally("population", alias="population"),
        "dem_2016": Tally("dem_2016", alias="dem_2016"),
        "rep_2016": Tally("rep_2016", alias="rep_2016"),
        "dem_2020": Tally("dem_2020", alias="dem_2020"),
        "rep_2020": Tally("rep_2020", alias="rep_2020"),
        "cut_edges": cut_edges,
    }
    initial = Partition(graph, assignment=assignment, updaters=updaters)
    ideal = sum(population) / k
    baseline = {
        "step": 0,
        "accepted": True,
        "cut_edges": len(initial["cut_edges"]),
        "cut_fraction": len(initial["cut_edges"]) / graph.number_of_edges(),
        "pop_deviation": max_deviation(initial, ideal, k),
        "democratic_seats_2016": democratic_seats(initial, "2016", k),
        "democratic_seats_2020": democratic_seats(initial, "2020", k),
    }

    traces = []
    proposal = partial(
        recom,
        pop_col="population",
        pop_target=ideal,
        epsilon=args.tolerance,
        node_repeats=10,
    )
    for chain_idx in range(args.chains):
        seed = chain_seed(args.base_seed, chain_idx)
        random.seed(seed)
        np.random.seed(seed & 0xFFFFFFFF)
        chain = MarkovChain(
            proposal=proposal,
            constraints=[within_percent_of_ideal_population(initial, args.tolerance)],
            accept=always_accept,
            initial_state=initial,
            total_steps=args.steps + 1,
        )
        metrics = []
        snapshots = []
        previous = None
        for step, partition in enumerate(chain):
            if step == 0:
                previous = dict(partition.assignment)
                continue
            current = dict(partition.assignment)
            accepted = current != previous
            metrics.append(
                {
                    "step": step,
                    "accepted": accepted,
                    "cut_edges": len(partition["cut_edges"]),
                    "cut_fraction": len(partition["cut_edges"]) / graph.number_of_edges(),
                    "pop_deviation": max_deviation(partition, ideal, k),
                    "democratic_seats_2016": democratic_seats(partition, "2016", k),
                    "democratic_seats_2020": democratic_seats(partition, "2020", k),
                }
            )
            if step % args.snapshot_stride == 0:
                snapshots.append(
                    {
                        "step": step,
                        "assignment": [int(current[node]) for node in range(n)],
                    }
                )
            previous = current
        traces.append(
            {
                "chain_idx": chain_idx,
                "seed": seed,
                "metrics": metrics,
                "snapshots": snapshots,
            }
        )

    output = {
        "trace_version": "g-real-gerrychain-trace v1",
        "implementation": "gerrychain",
        "implementation_version": getattr(gerrychain, "__version__", "unknown"),
        "tree_sampler": "gerrychain-random-weight-kruskal",
        "state": args.state.upper(),
        "steps_per_chain": args.steps,
        "chains": args.chains,
        "population_tolerance": args.tolerance,
        "base_seed": args.base_seed,
        "snapshot_stride": args.snapshot_stride,
        "adjacency_vertices": n,
        "adjacency_edges": graph.number_of_edges(),
        "election_input_version": elections["election_input_version"],
        "unmatched_geoids": elections["unmatched_geoids"],
        "baseline": baseline,
        "chain_traces": traces,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, separators=(",", ":")), encoding="utf-8")


if __name__ == "__main__":
    main()
