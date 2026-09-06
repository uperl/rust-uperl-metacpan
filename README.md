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
uperl-metacpan <SUBCOMMAND> [ARGS] [--json] [--raw] [--curl] [--color <auto|always|never>]
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
| `download <MODULE> [--version <RANGE>] [--dev]` | download that archive into the current directory and verify its SHA-256 |
| `mirrors` | known CPAN mirrors |
| `search --type <TYPE> --query <LUCENE> [--size N] [--from N]` | a Lucene query against a document type |
| `cache path` | print the cache directory |
| `cache status` | show the cache location, entry count, and disk space used (actual blocks allocated, like `du`) |
| `cache clear` | delete every cached response |

### Global options

- `-j`, `--json` — emit pretty-printed JSON instead of a table.
- `--color <auto\|always\|never>` — when to colour JSON output. `auto` (the
  default) colours only when stdout is a terminal, so piping or redirecting
  produces plain JSON.
- `--no-cache` — bypass the response cache for this run.
- `--cache-dir <DIR>` — override the cache directory.
- `--base-url <URL>` — talk to a private MetaCPAN deployment.
- `--raw` — instead of a table or JSON, print the raw HTTP request and response
  for each request the command makes: the request line and headers, a blank
  line, the response status line and headers, a blank line, then the body
  verbatim. The cache is bypassed, and `download` prints both the
  `download_url` lookup and the tarball fetch without writing a file. The
  exchange is printed even on failure, but a `4xx`/`5xx` response still exits
  non-zero. Does not apply to `cache` subcommands.
- `--curl` — print an equivalent `curl` command line and make no request.
  `download` and `download-url` print the `download_url` lookup; the tarball URL
  it resolves to is only known once that request runs. Does not apply to
  `cache` subcommands, and cannot be combined with `--raw`.

### Caching

Successful GET responses are cached on disk for one hour. The default location
is the platform cache directory:

| Platform | Default cache directory |
| --- | --- |
| Linux | `~/.cache/uperl/metacpan` (respects `$XDG_CACHE_HOME`) |
| macOS | `~/Library/Caches/uperl/metacpan` |
| Windows | `%LOCALAPPDATA%\uperl\metacpan` |

Check how much space it uses with `uperl-metacpan cache status`, clear it with
`uperl-metacpan cache clear`, or run any command with `--no-cache` to skip it.

### Examples

```sh
uperl-metacpan author PLICEASE
uperl-metacpan release FFI-Platypus
uperl-metacpan release FFI-Platypus-2.10 --author PLICEASE
uperl-metacpan module FFI::Platypus --json
uperl-metacpan search --type release --query "author:PLICEASE AND status:latest" --size 20
uperl-metacpan download-url FFI::Platypus --version "== 2.08"
uperl-metacpan download FFI::Platypus --version "== 2.08"
uperl-metacpan --json mirrors | jq '.[].name'
uperl-metacpan --raw author PLICEASE
uperl-metacpan --curl search --type release --query "author:PLICEASE" --size 5
uperl-metacpan cache clear
```

## License

MIT
