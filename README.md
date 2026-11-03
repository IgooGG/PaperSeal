# PaperSeal

PaperSeal is a small Rust application for generating A4 PDF sheets with a randomized fine-line background, a unique printed page ID, and an area for a physical stamp. It builds two Linux programs:

- `paperseal` — GTK desktop interface with an A4 preview.
- `paperseal-cli` — command-line PDF generator.

The pattern and stamp can discourage casual copying and help track issued pages. They cannot make a printed page impossible to scan, photocopy, or recreate, and the page ID is not cryptographic proof of origin.

## Requirements

- Linux (developed on Arch Linux)
- Rust and Cargo
- GTK 4 runtime and development libraries
- Cairo development library (normally installed as a GTK dependency)

On Arch Linux, install the packages with:

```sh
sudo pacman -S --needed rust gtk4 cairo
```

## Build

From the repository root:

```sh
cargo build --release
```

The executables are created at:

```text
target/release/paperseal
target/release/paperseal-cli
```

## Install

To install both commands into Cargo's user binary directory (`~/.cargo/bin`):

```sh
cargo install --path .
```

Make sure `~/.cargo/bin` is on your `PATH`. For example, add this to `~/.bashrc` if needed:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

Then open the GUI by running:

```sh
paperseal
```

## Graphical interface

The GUI provides a preview, a PDF filename field, and a page-count selector from 1 to 100. Choose **Generate A4 PDF** to create the file. For multi-page files, the preview displays the first generated page, including its actual randomized line pattern and page ID. Each other page has its own pattern and ID.

If the destination file already exists, PaperSeal asks before overwriting it. Cancel keeps the existing file.

## Command line

Create a single-page PDF:

```sh
paperseal-cli form.pdf
```

Create a multi-page PDF (the optional page count is 1–100):

```sh
paperseal-cli forms.pdf 12
```

If the output file already exists, the CLI warns and asks for confirmation. Enter `y` or `yes` to replace it; any other response cancels and leaves the existing file untouched.

The command prints the generated page IDs after success. Keep a record of those IDs if you plan to use them to track issued sheets.

## GitHub releases

The workflow in `.github/workflows/release.yml` builds Linux x86_64 binaries and uploads a tarball plus SHA-256 checksum to the matching GitHub release. The archive contains both executables and this README, plus the MIT license when it exists at the tagged revision. It runs when a version tag with or without a leading `v` is pushed to GitHub and can also be started manually from the GitHub Actions tab by entering an existing tag. The workflow uses GitHub's automatic `GITHUB_TOKEN`; the repository must allow Actions to write contents/releases. The release archive requires GTK 4 and Cairo runtime libraries for the GUI program.

## Printing and design notes

- The PDF page size is A4 (595 × 842 points).
- Print at **Actual Size** or **100%**; disable options such as “Fit to page” if they change the scale.
- The background uses bold vector strokes at 70% transparency (30% opacity), with wider spacing and no full-page solid fill.
- Printer model, driver, toner, paper, and print quality affect the result. Check a sample on the target printer before producing a batch.
- The stamp area is blank so a physical stamp can be applied after printing. A stamp impression can also be copied, so it should be checked as part of an issuer's process rather than treated as proof on its own.

## Randomness and limits

PaperSeal reads fresh bytes from Linux `/dev/urandom`, backed by the kernel's cryptographic random number generator. It uses these bytes to generate each page ID and vary the pattern. It does not collect mouse movement or retain a seed for reproducing a page.

The page ID is a random 12-character hexadecimal tracking label. It is not a digital signature, and the PDF does not authenticate its issuer. Stronger verification needs a trusted issuer and a record of which IDs were issued.

## Current scope

PaperSeal currently exports PDF only. PNG export and configurable page sizes are not implemented. The preview and PDF use the same generated line data.
