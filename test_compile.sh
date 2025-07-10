#!/bin/bash
cd /Users/r/git/glsp-rust
cargo build -p glsp-mcp-server 2>&1 | grep -E "(error|warning)" -A 3 -B 3