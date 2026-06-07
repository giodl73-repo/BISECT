#!/usr/bin/env python3
"""Build a bounded Track Y subset fixture from a RADJ adjacency file.

This script is intentionally research-scoped. It reads the native RADJ v2
format, selects a connected BFS neighborhood around a seed vertex, and emits
diagnostics needed before running the geographic/cohesion mode pair.
"""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import pickle
import struct
import sys
import tempfile
from collections import deque
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a Track Y connected subset fixture from RADJ adjacency."
    )
    parser.add_argument("--adjacency", type=Path)
    parser.add_argument("--seed-index", type=int, default=0)
    parser.add_argument("--scan-seeds", type=int, default=0)
    parser.add_argument("--max-vertices", type=int, default=250)
    parser.add_argument("--geoids", type=Path)
    parser.add_argument("--write-json", type=Path)
    parser.add_argument(
        "--allow-rejected",
        action="store_true",
        help="Return success after writing a rejected diagnostic fixture.",
    )
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def read_adjacency(path: Path) -> tuple[list[int], list[list[int]], dict[tuple[int, int], float]]:
    if path.suffix == ".pkl":
        return read_pkl(path)
    return read_radj(path)


def read_radj(path: Path) -> tuple[list[int], list[list[int]], dict[tuple[int, int], float]]:
    data = path.read_bytes()
    if len(data) < 16:
        raise ValueError(f"{path} is shorter than a RADJ header")
    magic, version, n_vertices, n_edges = struct.unpack_from("<4sIII", data, 0)
    if magic != b"RADJ":
        raise ValueError(f"{path} has magic {magic!r}, expected b'RADJ'")
    if version != 2:
        raise ValueError(f"{path} has RADJ version {version}, expected 2")

    pos = 16
    vertex_weights: list[int] = []
    for _ in range(n_vertices):
        (weight,) = struct.unpack_from("<q", data, pos)
        pos += 8
        vertex_weights.append(weight)

    adjacency: list[list[int]] = []
    for _ in range(n_vertices):
        (n_neighbors,) = struct.unpack_from("<I", data, pos)
        pos += 4
        neighbors = list(struct.unpack_from(f"<{n_neighbors}I", data, pos))
        pos += 4 * n_neighbors
        adjacency.append(neighbors)

    (n_weights,) = struct.unpack_from("<I", data, pos)
    pos += 4
    if n_weights != n_edges:
        raise ValueError(f"{path} declares {n_edges} edges but stores {n_weights} weights")

    edge_weights: dict[tuple[int, int], float] = {}
    for _ in range(n_weights):
        u, v, weight = struct.unpack_from("<IId", data, pos)
        pos += 16
        if u >= v:
            raise ValueError(f"{path} has non-canonical edge ({u}, {v})")
        edge_weights[(u, v)] = weight

    return vertex_weights, adjacency, edge_weights


def read_pkl(path: Path) -> tuple[list[int], list[list[int]], dict[tuple[int, int], float]]:
    with path.open("rb") as fh:
        raw = pickle.load(fh)
    if not isinstance(raw, dict):
        raise ValueError(f"{path} must contain a pickled dict")

    adjacency_raw = raw.get("adjacency")
    if adjacency_raw is None:
        adjacency_raw = raw.get("adj")
    if adjacency_raw is None:
        raise ValueError(f"{path} missing adjacency/adj key")

    adjacency = [list(map(int, row)) for row in adjacency_raw]
    vertex_weights_raw = raw.get("vertex_weights")
    if vertex_weights_raw is None:
        vertex_weights_raw = raw.get("populations")
    if vertex_weights_raw is None:
        vertex_weights = [1 for _ in adjacency]
    else:
        vertex_weights = [int(value) for value in vertex_weights_raw]
    if len(vertex_weights) != len(adjacency):
        raise ValueError(
            f"{path} has {len(vertex_weights)} vertex weights for {len(adjacency)} vertices"
        )

    edge_weights: dict[tuple[int, int], float] = {}
    raw_edge_weights = raw.get("edge_weights")
    if isinstance(raw_edge_weights, dict):
        for key, value in raw_edge_weights.items():
            if isinstance(key, str):
                parts = key.replace(",", " ").replace("-", " ").split()
                if len(parts) != 2:
                    continue
                u, v = int(parts[0]), int(parts[1])
            else:
                u, v = key
            if u != v:
                edge_weights[(u, v) if u < v else (v, u)] = float(value)

    if not edge_weights:
        for u, neighbors in enumerate(adjacency):
            for v in neighbors:
                if u != v:
                    edge_weights[(u, v) if u < v else (v, u)] = 1.0

    return vertex_weights, adjacency, edge_weights


def load_geoids(path: Path | None) -> list[str] | None:
    if path is None or not path.exists():
        return None
    raw = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(raw, list):
        return [str(value) for value in raw]
    if isinstance(raw, dict):
        return [str(raw[str(i)]) for i in range(len(raw))]
    raise ValueError(f"{path} must contain a list or index-keyed object")


def select_subset(adjacency: list[list[int]], seed: int, max_vertices: int) -> list[int]:
    if seed < 0 or seed >= len(adjacency):
        raise ValueError(f"seed index {seed} is outside graph with {len(adjacency)} vertices")
    if max_vertices <= 0:
        raise ValueError("--max-vertices must be positive")

    selected: list[int] = []
    seen = {seed}
    queue: deque[int] = deque([seed])
    while queue and len(selected) < max_vertices:
        node = queue.popleft()
        selected.append(node)
        for neighbor in sorted(adjacency[node], key=lambda n: (-len(adjacency[n]), n)):
            if neighbor not in seen:
                seen.add(neighbor)
                queue.append(neighbor)
    return selected


def induced_edges(adjacency: list[list[int]], subset: set[int]) -> list[tuple[int, int]]:
    edges: set[tuple[int, int]] = set()
    for u in subset:
        for v in adjacency[u]:
            if v in subset and u != v:
                edges.add((u, v) if u < v else (v, u))
    return sorted(edges)


def count_triangles(edges: list[tuple[int, int]]) -> int:
    neighbors: dict[int, set[int]] = {}
    for u, v in edges:
        neighbors.setdefault(u, set()).add(v)
        neighbors.setdefault(v, set()).add(u)

    triangles = 0
    for u, v in edges:
        common = neighbors[u].intersection(neighbors[v])
        triangles += sum(1 for w in common if v < w)
    return triangles


def bridge_count(edges: list[tuple[int, int]], vertices: list[int]) -> int:
    graph = {vertex: [] for vertex in vertices}
    for u, v in edges:
        graph[u].append(v)
        graph[v].append(u)

    time = 0
    seen: set[int] = set()
    tin: dict[int, int] = {}
    low: dict[int, int] = {}
    bridges = 0

    def dfs(node: int, parent: int | None) -> None:
        nonlocal time, bridges
        seen.add(node)
        tin[node] = low[node] = time
        time += 1
        for next_node in graph[node]:
            if next_node == parent:
                continue
            if next_node in seen:
                low[node] = min(low[node], tin[next_node])
            else:
                dfs(next_node, node)
                low[node] = min(low[node], low[next_node])
                if low[next_node] > tin[node]:
                    bridges += 1

    for vertex in vertices:
        if vertex not in seen:
            dfs(vertex, None)
    return bridges


def build_fixture(
    adjacency_path: Path,
    geoids_path: Path | None,
    seed_index: int,
    max_vertices: int,
    scan_seed_count: int = 0,
) -> dict[str, object]:
    vertex_weights, adjacency, edge_weights = read_adjacency(adjacency_path)
    _ = edge_weights
    geoids = load_geoids(geoids_path)
    if scan_seed_count > 0:
        seed_index = choose_seed(adjacency, vertex_weights, max_vertices, scan_seed_count)
    subset_vertices = select_subset(adjacency, seed_index, max_vertices)
    subset = set(subset_vertices)
    edges = induced_edges(adjacency, subset)
    degrees = [sum(1 for neighbor in adjacency[v] if neighbor in subset) for v in subset_vertices]
    populations = [vertex_weights[v] for v in subset_vertices]
    total_population = sum(populations)
    bridge_edges = bridge_count(edges, subset_vertices)
    triangles = count_triangles(edges)
    has_cycle_evidence = triangles > 0 or len(edges) >= len(subset_vertices)
    has_bridge_evidence = bridge_edges > 0
    population_span = (max(populations) - min(populations)) if populations else 0
    accepted, acceptance_reasons = evaluate_acceptance(
        tract_count=len(subset_vertices),
        edge_count=len(edges),
        has_cycle_evidence=has_cycle_evidence,
        has_bridge_evidence=has_bridge_evidence,
        population_span=population_span,
    )

    return {
        "schema": "bisect.track_y.subset_fixture.v1",
        "adjacency_path": str(adjacency_path),
        "geoids_path": str(geoids_path) if geoids_path else None,
        "seed_index": seed_index,
        "scan_seed_count": scan_seed_count,
        "seed_geoid": geoids[seed_index] if geoids and seed_index < len(geoids) else None,
        "max_vertices": max_vertices,
        "tract_count": len(subset_vertices),
        "edge_count": len(edges),
        "total_population": total_population,
        "population_min": min(populations) if populations else 0,
        "population_max": max(populations) if populations else 0,
        "population_span": population_span,
        "degree_min": min(degrees) if degrees else 0,
        "degree_median": sorted(degrees)[len(degrees) // 2] if degrees else 0,
        "degree_max": max(degrees) if degrees else 0,
        "bridge_edge_count": bridge_edges,
        "triangle_count": triangles,
        "has_cycle_evidence": has_cycle_evidence,
        "has_bridge_evidence": has_bridge_evidence,
        "accepted_for_pulse_05": accepted,
        "acceptance_reasons": acceptance_reasons,
        "next_step": "run_mode_pair" if accepted else "adjust_subset_selection",
        "subset_indices": subset_vertices,
        "subset_geoids": [geoids[v] for v in subset_vertices] if geoids else None,
        "claim_boundary": "research fixture only; not a statewide, legal, or fairness claim",
    }


def evaluate_acceptance(
    tract_count: int,
    edge_count: int,
    has_cycle_evidence: bool,
    has_bridge_evidence: bool,
    population_span: int,
) -> tuple[bool, list[str]]:
    reasons: list[str] = []
    if tract_count < 10:
        reasons.append("tract_count below recommended minimum of 10")
    if edge_count < tract_count:
        reasons.append("edge_count below tract_count; cycle evidence may be weak")
    if not has_cycle_evidence:
        reasons.append("missing cycle evidence")
    if not has_bridge_evidence:
        reasons.append("missing bridge evidence")
    if population_span <= 0:
        reasons.append("missing population variance")
    return len(reasons) == 0, reasons


def choose_seed(
    adjacency: list[list[int]],
    vertex_weights: list[int],
    max_vertices: int,
    scan_seed_count: int,
) -> int:
    best_seed = 0
    best_score: tuple[float, int] | None = None
    limit = min(len(adjacency), scan_seed_count)
    for seed in range(limit):
        subset_vertices = select_subset(adjacency, seed, max_vertices)
        subset = set(subset_vertices)
        edges = induced_edges(adjacency, subset)
        triangles = count_triangles(edges)
        bridges = bridge_count(edges, subset_vertices)
        populations = [vertex_weights[v] for v in subset_vertices]
        population_span = max(populations) - min(populations) if populations else 0
        degree_span = (
            max(len(adjacency[v]) for v in subset_vertices)
            - min(len(adjacency[v]) for v in subset_vertices)
            if subset_vertices
            else 0
        )
        score = (
            float(triangles > 0) * 1_000_000.0
            + float(bridges > 0) * 1_000_000.0
            + min(population_span, 1_000_000)
            + degree_span
            + len(edges) / max(len(subset_vertices), 1)
        )
        ranked = (score, -seed)
        if best_score is None or ranked > best_score:
            best_score = ranked
            best_seed = seed
    return best_seed


def write_synthetic_radj(path: Path) -> None:
    vertex_weights = [100, 150, 200, 75, 50]
    adjacency = [
        [1, 2],
        [0, 2],
        [0, 1, 3],
        [2, 4],
        [3],
    ]
    edge_weights = {
        (0, 1): 10.0,
        (0, 2): 11.0,
        (1, 2): 12.0,
        (2, 3): 2.0,
        (3, 4): 1.0,
    }
    payload = bytearray()
    payload.extend(struct.pack("<4sIII", b"RADJ", 2, len(vertex_weights), len(edge_weights)))
    for weight in vertex_weights:
        payload.extend(struct.pack("<q", weight))
    for neighbors in adjacency:
        payload.extend(struct.pack("<I", len(neighbors)))
        for neighbor in neighbors:
            payload.extend(struct.pack("<I", neighbor))
    payload.extend(struct.pack("<I", len(edge_weights)))
    for (u, v), weight in sorted(edge_weights.items()):
        payload.extend(struct.pack("<IId", u, v, weight))
    path.write_bytes(payload)


def write_synthetic_pkl(path: Path) -> None:
    with path.open("wb") as fh:
        pickle.dump(
            {
                "vertex_weights": [100, 150, 200, 75, 50],
                "adjacency": [
                    [1, 2],
                    [0, 2],
                    [0, 1, 3],
                    [2, 4],
                    [3],
                ],
                "edge_weights": {
                    (0, 1): 10.0,
                    (0, 2): 11.0,
                    (1, 2): 12.0,
                    (2, 3): 2.0,
                    (3, 4): 1.0,
                },
            },
            fh,
        )


def self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "synthetic.adj.bin"
        write_synthetic_radj(path)
        fixture = build_fixture(path, None, seed_index=0, max_vertices=5)
        assert fixture["tract_count"] == 5
        assert fixture["edge_count"] == 5
        assert fixture["total_population"] == 575
        assert fixture["triangle_count"] == 1
        assert fixture["bridge_edge_count"] == 2
        assert fixture["has_cycle_evidence"] is True
        assert fixture["has_bridge_evidence"] is True
        assert fixture["accepted_for_pulse_05"] is False
        assert "tract_count below recommended minimum of 10" in fixture["acceptance_reasons"]
        assert fixture["next_step"] == "adjust_subset_selection"

        pkl_path = Path(tmp) / "synthetic.pkl"
        write_synthetic_pkl(pkl_path)
        pkl_fixture = build_fixture(pkl_path, None, seed_index=0, max_vertices=5)
        assert pkl_fixture["tract_count"] == 5
        assert pkl_fixture["edge_count"] == 5
        assert pkl_fixture["total_population"] == 575
        assert pkl_fixture["triangle_count"] == 1
        assert pkl_fixture["bridge_edge_count"] == 2

        scanned_fixture = build_fixture(path, None, seed_index=4, max_vertices=5, scan_seed_count=5)
        assert scanned_fixture["seed_index"] == 0
        assert scanned_fixture["scan_seed_count"] == 5
        assert scanned_fixture["has_cycle_evidence"] is True
        assert scanned_fixture["has_bridge_evidence"] is True

        original_argv = sys.argv
        try:
            out_path = Path(tmp) / "fixture.json"
            sys.argv = [
                "track_y_build_subset_fixture.py",
                "--adjacency",
                str(path),
                "--write-json",
                str(out_path),
                "--allow-rejected",
            ]
            with contextlib.redirect_stdout(io.StringIO()):
                assert main() == 0
            written = json.loads(out_path.read_text(encoding="utf-8"))
            assert written["accepted_for_pulse_05"] is False
            assert written["next_step"] == "adjust_subset_selection"
        finally:
            sys.argv = original_argv


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        print("[OK] track_y_build_subset_fixture self-test passed")
        return 0
    if args.adjacency is None:
        raise SystemExit("--adjacency is required unless --self-test is set")
    if args.write_json is None:
        raise SystemExit("--write-json is required unless --self-test is set")

    fixture = build_fixture(
        args.adjacency,
        args.geoids,
        args.seed_index,
        args.max_vertices,
        args.scan_seeds,
    )
    text = json.dumps(fixture, indent=2, sort_keys=True)
    args.write_json.parent.mkdir(parents=True, exist_ok=True)
    args.write_json.write_text(text + "\n", encoding="utf-8")
    print(text)
    return 0 if fixture["accepted_for_pulse_05"] or args.allow_rejected else 2


if __name__ == "__main__":
    raise SystemExit(main())
