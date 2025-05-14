# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

zxing-rs-no-cmake is a Rust wrapper for the ZXing-CPP barcode library. It provides FFI bindings to the C++ ZXing library for reading and decoding various barcode formats without requiring CMake for building.

## Build and Development Commands

### Build the project
```bash
cargo build
```

### Run tests
```bash
cargo test
```

### Run with specific features
```bash
# Build with image support
cargo build --features image

# Build with bundled ZXing-CPP
cargo build --features bundled
```

### Run the example
```bash
# Requires an image path as an argument
cargo run --example main /path/to/barcode/image.png
```

## Architecture Overview

The project is structured as follows:

1. **C++ Integration**: The project uses bindgen and cc crates to generate Rust bindings for the ZXing-CPP library.
   - `build.rs` compiles the C++ source files from the zxing-cpp submodule
   - `src/bindings.rs` contains the generated FFI bindings

2. **Core Components**:
   - `ImageView`: Wrapper for ZXing's image representation
   - `ReaderOptions`: Configuration for barcode reading operations
   - `Barcode`: Represents a detected barcode with various metadata
   - `read_barcodes()`: Main function for scanning and reading barcodes

3. **Features**:
   - `image`: Enables integration with the Rust "image" crate
   - `bundled`: Uses the bundled ZXing-CPP code

## Useful Information

- The code compiles the necessary C++ files from the ZXing-CPP library directly, without using CMake.
- The project supports various barcode formats including QR code, EAN, Code128, etc.
- Error handling is done through a custom Error enum with thiserror integration.
- The API is designed to be idiomatic Rust while maintaining full functionality of the underlying C++ library.