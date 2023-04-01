# Generate Rust types from Teal defintinons
This is an S-tier project... if the S stood for shit

# Installation

1. Clone the repo (recursively to pull submodules)
1. Generate tree-sitter bindings with `tree-sitter generate` inside the `tree-sitter-teal` directory
1. Uncomment the `tree-sitter-teal/bindings/rust/build.rs` to also build the `scanner`
1. Build the project with `cargo build`
