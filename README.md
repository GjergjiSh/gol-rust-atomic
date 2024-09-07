# Design

## Caching

The game of life requires a "snapshot" of the current grid state for each generation.
Each thread must update the current grid state by applying the algorithm to the snapshot.
This is helpful as it means we can *relax our synchronization efforts*

## Safe rust

Threads take ownership of the data they access
Threads return ownership of the data they access (optional)

Q1: Can we pass ownership of slices to other threads

* Data races prevented
* Moving, cloning and/or copying data is not cheap



## Unsafe rust

Threads share data through smart pointers

This means that for n threads we would have to clone our grid 4 times. Once per each thread. And then reassemble the original based on the cache

We do not want to copy or move ownership of the grid cells to the generator threads.
Each thread has a holds a reference to a slice of cells from the grid.



1. Each type must implement clone in a safe and unsafe manner
2. The safe implementation uses a mutable reference to self
3. The unsafe implementation uses a shared reference to self

---
The grid is meant to be shared across multiple components as a static reference (TODO: Grid is not static).