# Bottle Thermodynamics Simulation

A real-time numerical simulation of heat transfer within a sealed water bottle containing ice, water, and air.

![Simulation Screenshot](https://s6.uupload.ir/files/2025-09-19_14_49_40_n4u.png)

## Overview

This project models the thermodynamic exchanges (conduction, convection) between the three phases of matter (ice, water, air) inside a typical sealed bottle. It was developed as a practical application of heat transfer principles, inspired by coursework in thermal sciences.

The simulation provides a visual representation of temperature gradients and phase change (melting) over time.

## Features

*   **Multi-phase System:** Simulates the interaction between solid ice, liquid water, and air.
*   **Real-time Physics:** Parameters and constants can be adjusted to see their immediate effect on the system.
*   **Cross-Platform:** Built with Rust and the Macroquad engine, it runs seamlessly on both Windows and Linux.
*   **Educational Focus:** The code is heavily commented to explain the physical models being implemented.

## Physics Model

The core simulation is based on solving the heat equation numerically within the domain. Key phenomena modeled include:

*   **Conductive Heat Transfer:** Between the ice, water, and the bottle walls.
*   **Convective Heat Transfer:** Modeled within the water and air phases using simplified effective conductivity.
*   **Phase Change:** The melting of ice is handled based on the net energy transfer at the ice-water boundary.
*   **Boundary Conditions:** The external temperature is set to a constant value, acting as a heat sink/source.

For a detailed explanation of the equations and numerical methods used, please refer to the comments in the main source file: `src/main.rs`.
## How to Run

You can run the simulation in one of two ways: by using a pre-built executable (easy) or by building from source (requires Rust).

### Option 1: Using Pre-built Executables (Recommended for most users)

Pre-compiled binaries for Windows and Linux are available on the [Releases page](https://github.com/AMJoshaghani/IceBottle-Simulation/releases).

1.  Go to the **Releases** section of this GitHub repository.
2.  Download the latest version for your operating system (e.g., `.exe` for Windows or `.AppImage` for Linux).
3.  Run the executable file inside (e.g., `IceBottle-Simulation.exe` on Windows, `IceBottle-Simulation.AppImage` on Linux).
    *   **On Linux:** You may need to give the file executable permissions first. You can do this in your file manager or by running `chmod +x IceBottle-Simulation.AppImage` in a terminal within the extracted folder.

### Option 2: Building from Source (Requires Rust)

If you have the Rust toolchain installed, you can build and run the project directly.

1.  **Install Rust:** The easiest way is to use [rustup](https://rustup.rs/).
2.  **Clone the repository:**
    ```bash
    git clone https://github.com/AMJoshaghani/IceBottle-Simulation.git
    cd IceBottle-Simulation
    ```
3.  **Run the simulation:**
    ```bash
    cargo run --release
    ```
    The `--release` flag is important for getting optimal performance.
    
## Dependencies

This project relies on the [Macroquad](https://macroquad.rs/) game engine, which Rust's package manager, Cargo, automatically handles. You do not need to install it separately.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing & Feedback

This is a work-in-progress project created for educational purposes. Feedback, issues, and suggestions are highly welcome! Please don't hesitate to open an issue on GitHub or reach out directly.
