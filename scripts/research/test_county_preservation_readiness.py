"""Failure-path tests for the repository-only readiness audit."""
import copy
import csv
import unittest
from analyze_county_preservation_readiness import INPUTS, ROOT, summarize


class ReadinessTests(unittest.TestCase):
    def setUp(self):
        with (ROOT / INPUTS[0]).open(newline='', encoding='utf-8') as handle:
            self.nodes = list(csv.DictReader(handle))
        with (ROOT / INPUTS[1]).open(newline='', encoding='utf-8') as handle:
            self.states = list(csv.DictReader(handle))

    def test_observed_winners_do_not_become_pre_boundary_diversity(self):
        result = summarize(self.nodes, self.states)
        self.assertEqual(result['nodes'], 385)  # 435 districts minus 50 States
        self.assertIsNone(result['pre_boundary_physical_partition_count'])

    def test_missing_node_rejected(self):
        with self.assertRaises(ValueError):
            summarize(self.nodes[:-1], self.states)

    def test_duplicate_node_rejected(self):
        nodes = self.nodes[:-1] + [self.nodes[0]]
        with self.assertRaises(ValueError):
            summarize(nodes, self.states)

    def test_impossible_count_order_rejected(self):
        nodes = copy.deepcopy(self.nodes)
        nodes[0]['minimum_deviation_cut_partitions'] = '100000'
        with self.assertRaises(ValueError):
            summarize(nodes, self.states)

    def test_non_preserving_row_rejected(self):
        self.nodes[0]['assignment_match'] = 'false'
        with self.assertRaises(ValueError):
            summarize(self.nodes, self.states)

    def test_changed_state_schedule_rejected(self):
        self.states[0]['districts'] = '2'
        with self.assertRaises(ValueError):
            summarize(self.nodes, self.states)


if __name__ == '__main__':
    unittest.main()
