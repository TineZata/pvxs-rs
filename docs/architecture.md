pvxs-rust-wrapper/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs # Entry point for the library
│   ├── bindings.rs
│   ├── wrapper/
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   ├── server.rs
│   │   ├── types.rs
│   │   └── utils.rs
│   └── tests/
│       └── integration.rs
├── src-cxx/
│   ├── pvxs_wrapper.cpp
│   └── ...
├── include/
│   ├── pvxs_wrapper.h
│   └── ...
├── third_party/
│   ├── pvxs/
│   │   ├── include/
│   │   │   └── pvxs/
│   │   │       ├── client.h
│   │   │       ├── server.h
│   │   │       └── ... (other PVXS headers)
│   │   └── lib/
│   │       ├── pvxs.dll          # Windows DLL
│   │       └── ...               # Other platform binaries if needed
├── scripts/
│   ├── copy_dlls.ps1             # PowerShell script for Windows
│   └── copy_dlls.sh              # Shell script for Unix-like systems
├── examples/
│   ├── client_example.rs
│   └── server_example.rs
├── docs/
│   ├── architecture.md
│   ├── contributing.md
│   └── usage.md
├── .gitignore
└── README.md
