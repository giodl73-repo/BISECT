import unittest

from run_ri_population_band import eligible, evaluate, tree_intervals
from test_dfs_county_response import path_context


class BandTests(unittest.TestCase):
    def test_integer_band_boundary(self):
        self.assertTrue(eligible(10, 1000000, 10))
        self.assertFalse(eligible(11, 1000000, 10))

    def test_all_tree_edges_not_only_population_minima(self):
        result = evaluate(path_context(), bands=(0, 1000000))
        self.assertEqual(result["bands"][0]["eligible_physical_cuts"], 1)
        self.assertEqual(result["bands"][1]["eligible_physical_cuts"], 9)
        self.assertEqual(len(result["candidates"]), 9)

    def test_empty_band_reported(self):
        context = path_context()
        context["populations"][0] += 1
        result = evaluate(context, bands=(0,))
        self.assertEqual(result["bands"][0]["status"], "empty-in-family")
        self.assertFalse(result["hypothesis_survives"])

    def test_county_scoring_changes_winning_set(self):
        context = path_context()
        context["units"]["unit_ids"][6:] = [f"01003{i:010d}" for i in range(6, 10)]
        result = evaluate(context, bands=(1000000,))
        self.assertTrue(result["hypothesis_survives"])
        winner = result["bands"][0]["alpha_response"][-1]["winners"]
        self.assertEqual(len(winner), 1)
        self.assertEqual(result["candidates"][winner[0]]["split_counties"], 0)

    def test_nonzero_root_subtree_populations(self):
        context = path_context()
        entry, leave, subtree = tree_intervals(context["graph"]["adjacency"], context["populations"], 5)
        self.assertEqual(subtree[5], 1000)
        for u in range(10):
            expected = sum(p for v, p in enumerate(context["populations"])
                           if entry[u] <= entry[v] < leave[u])
            self.assertEqual(subtree[u], expected)


if __name__ == "__main__":
    unittest.main()
