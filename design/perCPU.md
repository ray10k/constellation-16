PerCPU
===

The main implementation of the DCPU-16 processor, and the beating heart of the constellation-16 virtual machine. This is the main processor of the system.

## Decisions
1. Where possible, a clock-speed of 100kHz with accurate cycle-timings is the target.
2. The CPU is not in and of itself responsible for managing memory (to make ownership more manageable.)
3. The "catch on fire" mechanic will not be implemented.