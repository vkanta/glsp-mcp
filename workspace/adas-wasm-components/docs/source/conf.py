# Configuration file for the ADAS WASM Components documentation
# This extends the core GLSP-Rust documentation with workspace-specific requirements

import os
import sys

# -- Project information -----------------------------------------------------
project = 'ADAS WASM Components'
copyright = '2024, GLSP-Rust Team'
author = 'GLSP-Rust Team'
release = '0.1.0'

# -- General configuration ---------------------------------------------------
extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.intersphinx',
    'sphinx.ext.todo',
    'sphinx_needs',
    'myst_parser',
    'sphinxcontrib.plantuml',
    'sphinx_design',
]

# -- Sphinx-needs configuration ----------------------------------------------
# Import needs types from main documentation
needs_types = [
    {
        'directive': 'adas_req',
        'title': 'ADAS Requirement',
        'prefix': 'ADAS_REQ_',
        'color': '#FF5722',
        'style': 'node'
    },
    {
        'directive': 'adas_comp',
        'title': 'ADAS Component',
        'prefix': 'ADAS_COMP_',
        'color': '#FF9800',
        'style': 'node'
    },
    {
        'directive': 'adas_safety',
        'title': 'ADAS Safety Requirement',
        'prefix': 'ADAS_SAFETY_',
        'color': '#F44336',
        'style': 'node'
    },
    {
        'directive': 'sensor_req',
        'title': 'Sensor Requirement',
        'prefix': 'SENSOR_',
        'color': '#03A9F4',
        'style': 'node'
    },
    {
        'directive': 'ai_comp',
        'title': 'AI Component',
        'prefix': 'AI_COMP_',
        'color': '#9C27B0',
        'style': 'node'
    }
]

needs_extra_options = [
    'asil_level',          # Automotive Safety Integrity Level
    'sensor_type',         # Type of sensor (camera, lidar, radar, etc.)
    'ai_model',           # AI model used (YOLOv5n, etc.)
    'latency_requirement', # Performance latency requirement
    'component_category',  # Category of component (sensor, fusion, control, etc.)
    'iso_reference',      # ISO 26262 reference
    'wit_interface',      # WIT interface file reference
    'bazel_target',       # Bazel build target
]

# Link to main documentation
intersphinx_mapping = {
    'glsp-rust': ('../../../../docs/build/html', None),
}

# -- Options for HTML output -------------------------------------------------
html_theme = 'furo'
html_static_path = ['_static']
html_title = 'ADAS WASM Components Documentation'

# Theme options
html_theme_options = {
    'sidebar_hide_name': False,
    'navigation_with_keys': True,
}