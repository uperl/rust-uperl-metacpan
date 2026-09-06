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
| `river distribution <DIST> [--by total\|immediate] [--reverse] [--limit N]` | the distribution's direct reverse dependencies (with each one's latest-release author), ordered by CPAN River total |
| `river author <PAUSEID> [--by total\|immediate] [--reverse] [--limit N]` | the distributions whose current (latest, non-dev) release is by that author, ordered by CPAN River total |
| `permissions module <MODULE>...` | PAUSE upload permissions (`06perms`) for one or more module namespaces |
| `permissions author <PAUSEID> [--owner] [--comaint]` | every module namespace a PAUSE id owns or co-maintains |
| `adoptable [--by total\|immediate] [--reverse] [--limit N]` | distributions up for adoption (ADOPTME / HANDOFF permissions), with River figures, most-depended-on first |
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

Successful responses are cached on disk for one hour — every GET, and the
`_search` POSTs that back `river` and `adoptable` too (keyed by URL and request
body). The default location is the platform cache directory:

| Platform | Default cache directory |
| --- | --- |
| Linux | `~/.cache/uperl/metacpan` (respects `$XDG_CACHE_HOME`) |
| macOS | `~/Library/Caches/uperl/metacpan` |
| Windows | `%LOCALAPPDATA%\uperl\metacpan` |

Check how much space it uses with `uperl-metacpan cache status`, clear it with
`uperl-metacpan cache clear`, or run any command with `--no-cache` to skip it.

### River

`river distribution <DIST>` lists the distributions whose latest release
depends directly on `<DIST>`, ordered by their CPAN River total (transitive
downstream count), largest first. Pass `--by immediate` to rank on the direct
dependent count instead, `--limit N` to keep only the top N, and `--reverse` to
show that list smallest-first (with `--limit`, still the top N — just flipped).
Each row also
shows the `author` (PAUSE id) of that distribution's most recent production
release. It pages through the reverse-dependency list and then looks up the
River figures for those distributions, so it makes several requests; every
response is cached, so a re-run within the hour is fast. A distribution with no
River data on MetaCPAN is still listed, with `-` for its figures.

MetaCPAN's `reverse_dependencies` endpoint only serves its first ~900 results;
for a distribution with more reverse dependencies than that, the command prints
a note on stderr and lists the 900 it could retrieve.

`river author <PAUSEID>` lists the distributions whose current (latest,
non-dev) release was uploaded by that author — a view of which of an author's
distributions sit highest up the river. It takes the same `--by`, `--reverse`,
and `--limit` options; the `author` column is omitted since every row is the
queried author.

### Permissions

`permissions module <MODULE>...` shows the PAUSE `06perms` entry — primary
`owner` and `co-maintainers` — for each namespace. One module is a direct
lookup (a namespace with no entry is an error); two or more are fetched in a
single `by_module` request and unknown namespaces are simply omitted.

`permissions author <PAUSEID>` lists every namespace that id owns or
co-maintains. `--owner` keeps only the ones it owns, `--comaint` only the ones
it co-maintains; passing both is the same as passing neither.

`adoptable` lists every distribution up for adoption — one with a current
release providing a namespace that the `ADOPTME` or `HANDOFF` pseudo-users own
or co-maintain — with its CPAN River `total` and `immediate`, largest `total`
first. A `pause id` column after the distribution name shows which of the two
applies (`ADOPTME`, `HANDOFF`, or `ADOPTME,HANDOFF`); it is `pauseid` in
`--json`. It takes the same
`--by total|immediate`, `--reverse`, and `--limit N` options as the `river`
subcommands. It reads both authors' permissions, resolves each namespace to the
distribution that currently provides it, and looks up River figures, so it
makes many requests — every response is cached, so the first run takes a few
seconds and re-runs are near-instant.

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
uperl-metacpan river distribution Try-Tiny
uperl-metacpan river author PLICEASE --by immediate
uperl-metacpan permissions module Moose Try::Tiny
uperl-metacpan permissions author PLICEASE --json
uperl-metacpan adoptable
uperl-metacpan --raw author PLICEASE
uperl-metacpan --curl search --type release --query "author:PLICEASE" --size 5
uperl-metacpan cache clear
```

## License

MIT
