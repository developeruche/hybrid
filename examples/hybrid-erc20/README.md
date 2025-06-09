# RISC-V ERC-20 with cargo-hybrid
This is an ERC20 implementation writen in RUST and deployed on hybrid-node

### Compiling contract
```sh
cargo hybrid build
```

### running development node
```sh
cargo hybrid node
```

### Deploying contract
```sh
cargo hybrid deploy --encoded-args 0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266
```

--encoded-args: This is the abi-encoding of the constructor args