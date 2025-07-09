# Configuration file for the Sphinx documentation builder.
#
# This file only contains a selection of the most common options. For a full
# list see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Path setup --------------------------------------------------------------

# If extensions (or modules to document with autodoc) are in another directory,
# add these directories to sys.path here. If the directory is relative to the
# documentation root, use os.path.abspath to make it absolute, like shown here.
#
import os
import sys
sys.path.insert(0, os.path.abspath('../../glsp-mcp-server/src'))

# -- Project information -----------------------------------------------------

project = 'GLSP-Rust'
copyright = '2024, GLSP-Rust Team'
author = 'GLSP-Rust Team'
release = '0.1.0'

# -- General configuration ---------------------------------------------------

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.viewcode',
    'sphinx.ext.napoleon',
    'sphinx.ext.intersphinx',
    'sphinx.ext.todo',
    'sphinx.ext.coverage',
    'sphinx.ext.mathjax',
    'sphinx.ext.ifconfig',
    'sphinx_needs',
    'myst_parser',
    'sphinxcontrib.plantuml',
    'sphinx_design',
    'sphinx_copybutton',
]

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = []

# -- Options for HTML output -------------------------------------------------

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
#
html_theme = 'furo'

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ['_static']

# -- Options for sphinx_needs -----------------------------------------------

# Define custom need types for GLSP-Rust components
needs_types = [
    {
        'directive': 'req',
        'title': 'Requirement',
        'prefix': 'REQ_',
        'color': '#BFD5E7',
        'style': 'node'
    },
    {
        'directive': 'spec',
        'title': 'Specification',
        'prefix': 'SPEC_',
        'color': '#FEDCD2',
        'style': 'node'
    },
    {
        'directive': 'impl',
        'title': 'Implementation',
        'prefix': 'IMPL_',
        'color': '#DF744A',
        'style': 'node'
    },
    {
        'directive': 'test',
        'title': 'Test Case',
        'prefix': 'TEST_',
        'color': '#DCB239',
        'style': 'node'
    },
    {
        'directive': 'arch',
        'title': 'Architecture',
        'prefix': 'ARCH_',
        'color': '#8BC34A',
        'style': 'node'
    },
    {
        'directive': 'comp',
        'title': 'Component',
        'prefix': 'COMP_',
        'color': '#4CAF50',
        'style': 'node'
    },
    {
        'directive': 'mcp_req',
        'title': 'MCP Requirement',
        'prefix': 'MCP_',
        'color': '#FF9800',
        'style': 'node'
    },
    {
        'directive': 'wasm_req',
        'title': 'WASM Requirement',
        'prefix': 'WASM_',
        'color': '#9C27B0',
        'style': 'node'
    },
    {
        'directive': 'ai_req',
        'title': 'AI Requirement',
        'prefix': 'AI_',
        'color': '#E91E63',
        'style': 'node'
    },
    {
        'directive': 'db_req',
        'title': 'Database Requirement',
        'prefix': 'DB_',
        'color': '#00BCD4',
        'style': 'node'
    },
    {
        'directive': 'sim_req',
        'title': 'Simulation Requirement',
        'prefix': 'SIM_',
        'color': '#795548',
        'style': 'node'
    },
    {
        'directive': 'ui_req',
        'title': 'UI Requirement',
        'prefix': 'UI_',
        'color': '#607D8B',
        'style': 'node'
    },
    {
        'directive': 'safety_req',
        'title': 'Safety Requirement',
        'prefix': 'SAFETY_',
        'color': '#F44336',
        'style': 'node'
    }
]

# Custom options for needs
needs_extra_options = [
    'rationale',
    'verification',
    'safety_impact',
    'component_type',
    'mcp_operation',
    'wasm_component',
    'ai_capability',
    'database_backend',
    'simulation_type',
    'ui_component',
    'risk_level'
]

# Configure need layouts
needs_layouts = {
    'req': {
        'grid': 'simple',
        'layout': {
            'head': ['<<meta("type_name")>>: **<<meta("title")>>** <<meta("id")>>'],
            'meta': ['**Status:** <<meta("status")>>', '**Priority:** <<meta("priority")>>'],
            'footer': ['**Rationale:** <<meta("rationale")>>', '**Verification:** <<meta("verification")>>']
        }
    },
    'safety': {
        'grid': 'simple',
        'layout': {
            'head': ['<<meta("type_name")>>: **<<meta("title")>>** <<meta("id")>>'],
            'meta': ['**Status:** <<meta("status")>>', '**Risk Level:** <<meta("risk_level")>>'],
            'footer': ['**Safety Impact:** <<meta("safety_impact")>>', '**Verification:** <<meta("verification")>>']
        }
    }
}

# Configure need filters
needs_filter_data = {
    'current_version': release,
    'current_date': '2024-01-01'
}

# Configure need warnings
needs_warnings = {
    'req_missing_title': True,
    'req_missing_content': True,
    'req_missing_id': True
}

# Configure need role
needs_role_need_template = '**{title}** ({id})'

# Configure need links
needs_flow_configs = {
    'req_flow': {
        'link_types': ['links', 'tests', 'implements'],
        'allowed_filters': ['status', 'priority', 'component_type']
    }
}

# -- Options for PlantUML ---------------------------------------------------

# PlantUML configuration
plantuml = 'java -jar plantuml.jar'
plantuml_output_format = 'svg'
plantuml_latex_output_format = 'pdf'

# Configure PlantUML to work with different systems
import subprocess
import platform

def setup_plantuml():
    """Setup PlantUML based on the operating system."""
    if platform.system() == 'Windows':
        return 'plantuml.bat'
    elif platform.system() == 'Darwin':  # macOS
        try:
            subprocess.run(['which', 'plantuml'], check=True, capture_output=True)
            return 'plantuml'
        except subprocess.CalledProcessError:
            return 'java -jar /usr/local/bin/plantuml.jar'
    else:  # Linux
        try:
            subprocess.run(['which', 'plantuml'], check=True, capture_output=True)
            return 'plantuml'
        except subprocess.CalledProcessError:
            return 'java -jar /usr/bin/plantuml.jar'

plantuml = setup_plantuml()

# -- Options for MyST Parser -----------------------------------------------

# MyST Parser configuration
myst_enable_extensions = [
    'amsmath',
    'colon_fence',
    'deflist',
    'dollarmath',
    'fieldlist',
    'html_admonition',
    'html_image',
    'replacements',
    'smartquotes',
    'strikethrough',
    'substitution',
    'tasklist',
]

# -- Options for intersphinx ------------------------------------------------

intersphinx_mapping = {
    'python': ('https://docs.python.org/3/', None),
    'rust': ('https://doc.rust-lang.org/stable/', None),
}

# -- Options for todo extension ---------------------------------------------

todo_include_todos = True
todo_emit_warnings = True

# -- Options for coverage extension -----------------------------------------

coverage_show_missing_items = True
coverage_ignore_functions = ['main']

# -- HTML theme options -----------------------------------------------------

html_theme_options = {
    'sidebar_hide_name': True,
    'navigation_with_keys': True,
    'top_of_page_button': 'edit',
    'source_repository': 'https://github.com/glsp-rust/glsp-rust',
    'source_branch': 'main',
    'source_directory': 'docs/source/',
}

# -- Additional configuration -----------------------------------------------

# Add custom CSS
html_css_files = [
    'custom.css',
]

# Configure code highlighting
pygments_style = 'sphinx'
highlight_language = 'rust'

# Configure section numbering
numfig = True
numfig_format = {
    'figure': 'Figure %s',
    'table': 'Table %s',
    'code-block': 'Listing %s',
    'section': 'Section %s',
}

# Configure cross-references
autosectionlabel_prefix_document = True

# Configure copybutton
copybutton_prompt_text = r'>>> |\.\.\. |\$ |In \[\d*\]: | {2,5}\.\.\.: | {5,8}: '
copybutton_prompt_is_regexp = True
copybutton_only_copy_prompt_lines = True
copybutton_remove_prompts = True

# Configure version switcher
html_context = {
    'display_github': True,
    'github_user': 'glsp-rust',
    'github_repo': 'glsp-rust',
    'github_version': 'main',
    'conf_py_path': '/docs/source/',
}