import unittest
from run_ri_boundary_moves import connected, improve
from test_dfs_county_response import path_context


class BoundaryMoveTests(unittest.TestCase):
    def test_articulation_move_is_rejected(self):
        context = {"units": {"unit_ids": [f"01001{i:010d}" for i in range(4)]},
                   "populations": [100] * 4,
                   "graph": {"adjacency": [
                       [{"to": 1, "weight": 1}],
                       [{"to": 0, "weight": 1}, {"to": 2, "weight": 1}, {"to": 3, "weight": 10}],
                       [{"to": 1, "weight": 1}], [{"to": 1, "weight": 10}]]}}
        result = improve(context, [0, 0, 0, 1], "0", band=1000000)
        self.assertEqual(result["moves"], 0)
        self.assertGreater(result["connectivity_checks"], 0)
        self.assertEqual(result["assignment"], [0, 0, 0, 1])

    def test_zero_move_budget_is_not_convergence(self):
        result = improve(path_context(), [0] * 5 + [1] * 5, "0", move_budget=0)
        self.assertEqual(result["status"], "move-budget")
        self.assertEqual(result["moves"], 0)

    def test_empty_and_disconnected_labels_rejected(self):
        adjacency = [[(1, 1)], [(0, 1), (2, 1)], [(1, 1)]]
        self.assertFalse(connected(adjacency, [0, 0, 0], 1))
        self.assertFalse(connected(adjacency, [0, 1, 0], 0))

    def test_invalid_seed_rejected(self):
        with self.assertRaisesRegex(ValueError, "seed"):
            improve(path_context(), [0] * 10, "0")

    def test_no_strict_improvement_on_equal_path(self):
        result = improve(path_context(), [0] * 5 + [1] * 5, "2", band=1000000)
        self.assertEqual(result["moves"], 0)
        self.assertEqual(result["status"], "no-improving-feasible-single-vertex-move")

    def test_move_and_full_score_reconciliation(self):
        context = path_context()
        for u in (4, 5):
            for edge in context["graph"]["adjacency"][u]:
                if {u, edge["to"]} == {4, 5}:
                    edge["weight"] = 10
        result = improve(context, [0] * 5 + [1] * 5, "0.5", band=1000000)
        self.assertEqual(result["moves"], 1)
        self.assertEqual(result["final"]["raw_cut"], 1)
        self.assertTrue(result["final_connected"])

    def test_check_budget_preserves_seed(self):
        context = path_context()
        context["graph"]["adjacency"][4][1]["weight"] = 10
        result = improve(context, [0] * 5 + [1] * 5, "0", check_budget=0, band=1000000)
        self.assertEqual(result["status"], "connectivity-check-budget")
        self.assertEqual(result["moves"], 0)


if __name__ == "__main__":
    unittest.main()
