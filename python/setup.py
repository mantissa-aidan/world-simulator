from setuptools import setup
from setuptools_rust import RustExtension, Binding

setup(
    name="world-sim",
    version="0.1.0",
    packages=["world_sim"],
    rust_extensions=[
        RustExtension(
            "world_sim.world_simulator",
            path="../Cargo.toml",
            binding=Binding.PyO3,
            debug=False
        )
    ],
    install_requires=[
        "numpy>=1.20.0",
        "gymnasium>=0.26.0",  # For future RL integration
    ],
    setup_requires=[
        "setuptools-rust>=1.5.2",
    ],
    python_requires=">=3.7",
    zip_safe=False,  # Required for Rust extensions
    include_package_data=True,
    author="World Simulator Team",
    author_email="",
    description="A predator-prey simulation environment for reinforcement learning",
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    keywords="simulation, reinforcement-learning, predator-prey",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Science/Research",
        "Programming Language :: Python :: 3",
        "Programming Language :: Rust",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
    ],
) 