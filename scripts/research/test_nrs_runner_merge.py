"""Regression checks for merged state configuration and lexical input paths."""
import os
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

import run_nrs_bakeoff_national as tier1
import run_nrs_bakeoff_geometry_national as tier2
from state_config import STATE_CONFIG_2020


class RunnerMergeTests(unittest.TestCase):
    def test_both_runners_share_central_state_configuration(self):
        self.assertIs(tier1.STATE_CONFIG_2020, STATE_CONFIG_2020)
        self.assertIs(tier2.STATE_CONFIG_2020, STATE_CONFIG_2020)
        self.assertEqual(len(STATE_CONFIG_2020), 50)
        self.assertEqual(sum(row["districts"] for row in STATE_CONFIG_2020.values()), 435)

    def test_tier1_cli_keeps_lexical_paths(self):
        with patch.object(sys, "argv", ["runner", "--nrs-root", "runs/vault-link", "--output-dir", "target/merge-check"]), \
             patch.object(tier1, "run_national") as run, \
             patch.object(Path, "resolve", side_effect=AssertionError("must not dereference junction")):
            tier1.main()
        run.assert_called_once_with(Path(os.path.abspath("runs/vault-link")),
                                    Path(os.path.abspath("target/merge-check")))

    def test_tier2_cli_retains_display_path(self):
        with patch.object(sys, "argv", ["runner", "--nrs-root", "runs/vault-link", "--output-dir", "target/merge-check", "--display-output-dir", "docs/result"]), \
             patch.object(tier2, "run_national") as run:
            tier2.main()
        run.assert_called_once_with(Path("runs/vault-link"), Path("target/merge-check"), "docs/result")


if __name__ == "__main__":
    unittest.main()
