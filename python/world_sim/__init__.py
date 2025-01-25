"""
World Simulator Python Package

This package provides Python bindings for the Rust-based world simulator,
enabling reinforcement learning applications.
"""

from .world_simulator import PyWorld
from .env import WorldEnv, WorldSimEnv

__version__ = "0.1.0"
__all__ = ["PyWorld", "WorldEnv", "WorldSimEnv"] 