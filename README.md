# ROM Analyzer

A command-line tool written in Rust that analyzes ROM files (including those in `.zip` and `.chd` archives) to identify their region based on file headers.

## Features

*   **Region Identification:** Accurately determines the geographical region of various game ROMs.
*   **Archive Support:** Supports analysis of ROMs within `.zip` and `.chd` archives.
*   **Wide Console Support:** Compatible with a broad range of classic gaming console ROMs.

## Supported Consoles

The `rom-analyzer` currently supports ROMs for the following systems:

*   Game Boy (GB)
*   Game Boy Advance (GBA)
*   Game Gear
*   Master System
*   Nintendo 64 (N64)
*   Nintendo Entertainment System (NES)
*   PlayStation (PSX)
*   Sega CD
*   Sega Cartridge (general)
*   Super Nintendo Entertainment System (SNES)

## Installation

To build and install `rom-analyzer`, you'll need [Rust](https://www.rust-lang.org/tools/install) installed on your system.

```bash
git clone https://github.com/your-repo/rom-analyzer.git
cd rom-analyzer
cargo build --release
cargo install --path .
```

This will compile the project and install the `rom-analyzer` executable to your Cargo bin directory, making it available in your shell's PATH.

## Usage

To analyze a ROM file or an archive containing ROMs, simply run:

```bash
rom-analyzer <path-to-rom-or-archive>
```

**Example:**

```bash
rom-analyzer "MyGame (USA).zip"
rom-analyzer "AnotherGame.chd"
rom-analyzer "SingleRom.nes"
```

The tool will output the identified region for the ROMs found.

## Contributing

Contributions are welcome! Please feel free to open issues or submit pull requests.

## License

This project is licensed under the MIT License.
