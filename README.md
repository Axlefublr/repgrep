[![crate](https://img.shields.io/crates/v/repgrep)](https://crates.io/crates/repgrep)
[![documentation](https://docs.rs/repgrep/badge.svg)](https://docs.rs/repgrep)
[![Average time to resolve an issue](https://isitmaintained.com/badge/resolution/acheronfail/repgrep.svg)](https://isitmaintained.com/project/acheronfail/repgrep "Average time to resolve an issue")
[![Percentage of issues still open](https://isitmaintained.com/badge/open/acheronfail/repgrep.svg)](https://isitmaintained.com/project/acheronfail/repgrep "Percentage of issues still open")

# repgrep (rgr)

_An interactive replacer for `ripgrep`._

This is an interactive command line tool to make find and replacement easy.
It uses [`ripgrep`] to find, and then provides you with a simple interface to see
the replacements in real-time and conditionally replace matches.

Supported file encodings:

* ASCII
* UTF8
* UTF16BE
* UTF16LE

Other encodings are possibly supported but untested at the moment.
See [this issue](https://github.com/acheronfail/repgrep/issues/12) for more information.

## Usage

After installing, just use `rgr` (think: `rg` + `replace`).

The arguments are:

```bash
rgr <rg arguments> # See `rgr --help` for more details
```

Here's an example where we ran the command:

```bash
rgr -C5 dreamcast
```

And have entered the replacement `flycast`:

![demo using rgr](./doc/demo.png)

## Installation

First and foremost, make sure you've installed `ripgrep` (AKA: `rg`).
To do so see the [`ripgrep` installation instructions].

#### Precompiled binaries

See the [releases] page for pre-compiled binaries.

#### Via Cargo

**NOTE**: The minimum Rust version required is `1.46.0`.

```bash
cargo install repgrep
```

#### From Source (via Cargo)

**NOTE**: The minimum Rust version required is `1.46.0`.

```bash
git clone https://github.com/acheronfail/repgrep/
cd repgrep
cargo install --path .
```

[`ripgrep`]: https://github.com/BurntSushi/ripgrep
[releases]: https://github.com/acheronfail/repgrep/releases
[`ripgrep` installation instructions]: https://github.com/BurntSushi/ripgrep/#installation

License: Unlicense OR MIT OR Apache-2.0