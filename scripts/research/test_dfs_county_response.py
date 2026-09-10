import unittest

from analyze_dfs_county_response import analyze


def path_context():
    return {"units": {"unit_ids": [f"01001{i:010d}" for i in range(10)]},
            "populations": [100] * 10,
            "graph": {"adjacency": [[{"to": v, "weight": 1}
                                      for v in (u - 1, u + 1) if 0 <= v < 10]
                                     for u in range(10)]}}


class CountyResponseTests(unittest.TestCase):
    def test_complementary_orientations_are_one_cut(self):
        result = analyze(path_context(), 4)
        self.assertEqual(result["minimum_deviation_candidates"], 2)
        self.assertEqual(result["minimum_deviation_physical_partitions"], 1)
        self.assertFalse(result["winning_set_changed"])

    def test_distinct_minimum_population_cuts(self):
        result = analyze(path_context(), 5)
        self.assertEqual(result["minimum_deviation_candidates"], 2)
        self.assertEqual(result["minimum_deviation_physical_partitions"], 2)

    def test_county_weight_changes_winning_cut(self):
        context = path_context()
        # Population-optimal cuts are edges 3--4 and 5--6. The latter
        # crosses counties, costs 2 raw; the former is internal and costs 1.
        context["units"]["unit_ids"][6:] = [f"01003{i:010d}" for i in range(6, 10)]
        for u in (5, 6):
            for edge in context["graph"]["adjacency"][u]:
                if {u, edge["to"]} == {5, 6}:
                    edge["weight"] = 2
        result = analyze(context, 5)
        self.assertTrue(result["winning_set_changed"])
        self.assertEqual(len(result["alpha_response"][2]["winning_physical_partitions"]), 2)
        self.assertEqual(result["alpha_response"][3]["winning_physical_partitions"],
                         [context["units"]["unit_ids"][6]])

    def test_disconnected_rejected(self):
        context = path_context()
        context["graph"]["adjacency"][0] = []
        with self.assertRaisesRegex(ValueError, "disconnected"):
            analyze(context, 2)

    def test_noninteger_weights_rejected(self):
        context = path_context()
        context["graph"]["adjacency"][0][0]["weight"] = 0.5
        with self.assertRaisesRegex(ValueError, "integer"):
            analyze(context, 2)


if __name__ == "__main__":
    unittest.main()
