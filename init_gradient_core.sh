#!/bin/bash

# Initialize gradient-core as a separate git repository

cd ../gradient-core

# Initialize git repo
git init

# Add all files
git add .

# Initial commit
git commit -m "Initial commit: Gradient Core foundation library

- Mathematical primitives (tensors, optimization, statistics, calculus)
- Core algorithms (consensus, emergence detection, swarm intelligence, quantum)
- Communication protocols (network, serialization, messaging)
- Utilities (random, time, logging, collections)

Foundation library extracted from HAL9 project for reusability."

echo "Gradient Core repository initialized successfully!"
echo "Next steps:"
echo "1. Create a new repository on GitHub"
echo "2. Add remote: git remote add origin https://github.com/YOUR_USERNAME/gradient-core.git"
echo "3. Push: git push -u origin main"