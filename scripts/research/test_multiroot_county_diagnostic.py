import unittest

from run_multiroot_county_diagnostic import candidates, score_pool
from analyze_dfs_county_response import analyze
from test_dfs_county_response import path_context


class MultirootTests(unittest.TestCase):
    def test_root_zero_agrees_with_control_on_branched_graph(self):
        context = path_context()
        for u, v in ((0, 5), (2, 8), (3, 9)):
            context["graph"]["adjacency"][u].append({"to": v, "weight": 7})
            context["graph"]["adjacency"][v].append({"to": u, "weight": 7})
        control = analyze(context, 5)
        rows = candidates(context, 5, 0)
        self.assertEqual(score_pool(rows)["minimum_deviation"], control["minimum_deviation"])
        self.assertEqual(sorted((r["raw_cut"], r["within_county_cut"]) for r in rows),
                         sorted((r["raw_cut"], r["within_county_cut"]) for r in control["candidate_scores"]))

    def test_reversed_roots_deduplicate_same_physical_cut(self):
        context = path_context()
        left = candidates(context, 4, 0)
        right = candidates(context, 4, 9)
        self.assertEqual(left, right)
        self.assertEqual(score_pool(left + right)["physical_candidates_after_population_filter"], 1)

    def test_population_precedes_county_weights(self):
        rows = [dict(partition_sha256="a", minimum_deviation=1, raw_cut=100, within_county_cut=100),
                dict(partition_sha256="b", minimum_deviation=2, raw_cut=1, within_county_cut=0)]
        result = score_pool(rows)
        self.assertEqual(result["physical_candidates_before_population_filter"], 2)
        self.assertEqual(result["physical_candidates_after_population_filter"], 1)
        self.assertTrue(all(r["winners"] == ["a"] for r in result["alpha_response"]))

    def test_zero_population_units_remain_in_identity(self):
        context = path_context()
        context["populations"][4:6] = [0, 0]
        rows = candidates(context, 2, 0)
        self.assertEqual(len(rows), 3)
        self.assertEqual(len({r["partition_sha256"] for r in rows}), 3)


if __name__ == "__main__":
    unittest.main()
