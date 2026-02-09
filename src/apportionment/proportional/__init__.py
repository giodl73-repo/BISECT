"""
Proportional representation algorithms for seat allocation.

This module implements various proportional allocation methods used in
party-list PR systems internationally.
"""

from .dhondt import allocate_seats_dhondt, compute_dhondt_quotients

__all__ = ['allocate_seats_dhondt', 'compute_dhondt_quotients']
