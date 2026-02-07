# nl - Number Lines for Windows

POSIX `nl` command implemented in Rust. Drop-in replacement for GNU `nl` on Windows.

## Usage

```
nl [OPTION]... [FILE]
```

Reads from `FILE` or from stdin if no file is specified (or if FILE is `-`).

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `-b STYLE` | Body line numbering style | `t` |
| `-h STYLE` | Header line numbering style | `n` |
| `-f STYLE` | Footer line numbering style | `n` |
| `-d CC` | Section delimiter characters | `\:` |
| `-n FORMAT` | Line number format (`ln`, `rn`, `rz`) | `rn` |
| `-s STRING` | Separator between number and line | `TAB` |
| `-w NUMBER` | Line number field width | `6` |
| `-v NUMBER` | Starting line number | `1` |
| `-i NUMBER` | Line number increment | `1` |
| `-l NUMBER` | Group of N empty lines counted as one | `1` |
| `-p` | Do not reset line numbers for each section | |

### Numbering styles (STYLE)

- `a` — number all lines
- `t` — number only non-empty lines
- `n` — no numbering
- `pBRE` — number only lines matching the regular expression BRE

## Examples

```bash
# Number non-empty lines (default)
nl file.txt

# Number all lines including empty ones
nl -ba file.txt

# Leading zeros, custom width and separator
nl -nrz -w 4 -s ". " file.txt

# Number only lines containing "TODO"
nl -b "pTODO" file.txt

# From stdin
cat file.txt | nl
```

## Sections

Files can be divided into sections using delimiter lines:

- `\:\:\:` — start of header
- `\:\:` — start of body
- `\:` — start of footer

Each section can have its own numbering style (`-h`, `-b`, `-f`). Line numbers reset at each section boundary unless `-p` is specified.

## Build

```bash
cargo build --release
```

The binary will be at `target/release/nl.exe`.

## Live-coded with Claude Code

This project was live-coded with [Claude Code](https://claude.ai/claude-code) (Claude Opus 4.6) in a single session — from zero to a fully functional `nl` with all GNU options, tests, GitHub repo, and release.

## License

MIT
