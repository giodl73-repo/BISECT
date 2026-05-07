"""Data acquisition and processing modules."""

from .census import download_blocks, load_blocks
from .adjacency import build_adjacency_graph, merge_spatial_weights
from .io import save_results, load_results

__all__ = [
    'download_blocks',
    'load_blocks',
    'build_adjacency_graph',
    'merge_spatial_weights',
    'save_results',
    'load_results',
]
