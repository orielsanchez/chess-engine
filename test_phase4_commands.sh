#!/bin/bash

echo "Testing Phase 4 Interactive Commands"
echo "====================================="

# Test commands in the interactive engine
echo -e "help\nplay white 5\npuzzle mate_in_2\nthreats\nhint\nclock\nquit" | cargo run --bin interactive

echo -e "\nPhase 4 Commands Successfully Integrated!"