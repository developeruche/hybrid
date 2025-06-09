# Compile
This goal of this crate is to provide API for compiling a RUST based to smart contract to a RISC-V in way that is compatible with the RISCV sand box in  the node.

This crate is an adaptation of the work done on [r55]() some this of the code here is just a copy and paste. Going with this route because the compile crate in `r55` is a bin crate and it tightly coupled to their test cases. I an decoupling this to enjoy some form of flexiblity while building the POC.