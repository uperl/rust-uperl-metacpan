# uperl-metacpan

A command line interface to the [MetaCPAN](https://metacpan.org/) API, built on
the [`metacpan-api-modern`](https://github.com/uperl/rust-metacpan-api-modern)
crate.

Each MetaCPAN document type is a subcommand. Results print as a formatted table
by default; `--json` switches to pretty-printed JSON, syntax-coloured when
stdout is a terminal.

## Install

```sh
cargo install --path .
```

## Usage

```
uperl-metacpan <SUBCOMMAND> [ARGS] [--json] [--color <auto|always|never>]
```

| Subcommand | What it fetches |
| --- | --- |
| `author <PAUSEID>` | a CPAN author |
| `release <NAME> [--author <PAUSEID>]` | the latest release of a distribution, or a specific release |
| `module <MODULE>` | the file that provides a module |
| `file <AUTHOR> <RELEASE> <PATH>` | metadata for one file in a release |
| `source <AUTHOR> <RELEASE> <PATH>` | the raw source of one file (text) |
| `pod <MODULE> [--format <plain\|html\|markdown\|pod>]` | rendered documentation (text) |
| `distribution <DIST>` | distribution aggregates (CPAN River, bug counts, ...) |
| `changes <NAME> [--author <PAUSEID>]` | a distribution's change log |
| `download-url <MODULE> [--version <RANGE>] [--dev]` | the archive that satisfies a module request |
| `mirrors` | known CPAN mirrors |
| `search --type <TYPE> --query <LUCENE> [--size N] [--from N]` | a Lucene query against a document type |

### Global options

- `-j`, `--json` — emit pretty-printed JSON instead of a table.
- `--color <auto\|always\|never>` — when to colour JSON output. `auto` (the
  default) colours only when stdout is a terminal, so piping or redirecting
  produces plain JSON.
- `--cache-dir <DIR>` — cache successful GET responses on disk (1 hour TTL).
- `--base-url <URL>` — talk to a private MetaCPAN deployment.

### Examples

```sh
uperl-metacpan author PLICEASE
uperl-metacpan release FFI-Platypus
uperl-metacpan release FFI-Platypus-2.10 --author PLICEASE
uperl-metacpan module FFI::Platypus --json
uperl-metacpan search --type release --query "author:PLICEASE AND status:latest" --size 20
uperl-metacpan download-url FFI::Platypus --version "== 2.08"
uperl-metacpan --json mirrors | jq '.[].name'
```

## License

MIT
