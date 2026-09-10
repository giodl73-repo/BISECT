import unittest
from fractions import Fraction
from run_ri_adjacent_swaps import improve_swaps, swap_delta
from run_ri_boundary_moves import metrics


def fixture():
    edges = {(0, 1): 1, (2, 3): 1, (0, 2): 1, (0, 3): 10, (1, 2): 10, (1, 3): 1}
    graph = [[] for _ in range(4)]
    for (u, v), weight in edges.items():
        graph[u].append({"to": v, "weight": weight})
        graph[v].append({"to": u, "weight": weight})
    return {"units": {"unit_ids": [f"01001{u:010d}" for u in range(4)]},
            "populations": [100] * 4, "graph": {"adjacency": graph}}, edges


class SwapTests(unittest.TestCase):
    def test_equal_population_swap_escapes_single_move_band(self):
        context, _ = fixture()
        result = improve_swaps(context, [0, 0, 1, 1], "0.5", band=0)
        self.assertEqual(result["swaps"], 1)
        self.assertEqual(result["final"]["raw_cut"], 4)
        self.assertEqual(result["final"]["deviation_numerator"], 0)

    def test_pair_delta_equals_full_cost_difference(self):
        context, edges = fixture()
        labels = [0, 0, 1, 1]
        adjacency = [[(e["to"], e["weight"]) for e in row] for row in context["graph"]["adjacency"]]
        ids = context["units"]["unit_ids"]
        for u, v in edges:
            if labels[u] == labels[v]:
                continue
            changed = labels.copy()
            changed[u], changed[v] = changed[v], changed[u]
            old = metrics(ids, context["populations"], edges, labels, "0.5")
            new = metrics(ids, context["populations"], edges, changed, "0.5")
            self.assertEqual(Fraction(new["weighted_cut"]) - Fraction(old["weighted_cut"]),
                             Fraction(swap_delta(ids, adjacency, labels, u, v, "0.5"), 2))

    def test_check_budget_preserves_assignment(self):
        context, _ = fixture()
        result = improve_swaps(context, [0, 0, 1, 1], "0", check_budget=0, band=0)
        self.assertEqual(result["status"], "connectivity-check-budget")
        self.assertEqual(result["assignment"], [0, 0, 1, 1])

    def test_population_violation_rejected(self):
        context, _ = fixture()
        with self.assertRaisesRegex(ValueError, "infeasible seed"):
            improve_swaps(context, [0, 0, 0, 1], "0", band=0)


if __name__ == "__main__":
    unittest.main()
