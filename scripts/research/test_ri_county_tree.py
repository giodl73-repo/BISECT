import unittest
from run_ri_county_tree import spanning_tree, evaluate
from test_dfs_county_response import path_context


class CountyTreeTests(unittest.TestCase):
    def test_county_preference_changes_tree(self):
        ids = ["010010000000000", "010010000000001", "010030000000000"]
        edges = {(0, 1): 1, (0, 2): 10, (1, 2): 9}
        _, geographic = spanning_tree(ids, edges, False)
        _, county = spanning_tree(ids, edges, True)
        self.assertEqual(set(geographic), {(0, 2), (1, 2)})
        self.assertEqual(set(county), {(0, 1), (0, 2)})

    def test_original_graph_scoring_not_tree_scoring(self):
        context = path_context()
        context["graph"]["adjacency"][0].append({"to": 9, "weight": 2})
        context["graph"]["adjacency"][9].append({"to": 0, "weight": 2})
        result = evaluate(context, False, bands=(1000000,))
        # Every proper cut in a cycle crosses at least two original edges.
        self.assertTrue(all(r["raw_cut"] >= 2 for r in result["candidates"].values()))
        self.assertEqual(result["tree_edge_count"], 9)

    def test_order_independence(self):
        ids = ["010010000000000", "010010000000001", "010030000000000"]
        pairs = [((0, 1), 1), ((0, 2), 1), ((1, 2), 1)]
        self.assertEqual(spanning_tree(ids, dict(pairs), True),
                         spanning_tree(ids, dict(reversed(pairs)), True))

    def test_disconnected_rejected(self):
        with self.assertRaisesRegex(ValueError, "disconnected"):
            spanning_tree(["01001", "01003"], {}, True)


if __name__ == "__main__":
    unittest.main()
