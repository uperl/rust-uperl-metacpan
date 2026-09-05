//! `uperl-metacpan` — a command line interface to the MetaCPAN API.
//!
//! Each MetaCPAN document type is a subcommand. Results print as a formatted
//! table by default; `--json` switches to pretty-printed JSON, coloured when
//! stdout is a terminal (override with `--color`).

mod json;
mod render;

use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use metacpan_api_modern::{Client, PodFormat};
use serde_json::{Value, json};

/// Command line interface to the MetaCPAN API.
#[derive(Parser)]
#[command(name = "uperl-metacpan", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[command(flatten)]
    global: GlobalOpts,
}

#[derive(Args)]
struct GlobalOpts {
    /// Print pretty-printed JSON instead of a table.
    #[arg(long, short = 'j', global = true)]
    json: bool,

    /// When to colourise JSON output.
    #[arg(
        long,
        value_enum,
        default_value = "auto",
        global = true,
        value_name = "WHEN"
    )]
    color: ColorWhen,

    /// Cache successful GET responses under this directory.
    #[arg(long, global = true, value_name = "DIR")]
    cache_dir: Option<PathBuf>,

    /// Override the API base URL.
    #[arg(long, global = true, value_name = "URL")]
    base_url: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum ColorWhen {
    /// Colour when stdout is a terminal.
    Auto,
    /// Always colour.
    Always,
    /// Never colour.
    Never,
}

#[derive(Copy, Clone, ValueEnum)]
enum PodFmt {
    Plain,
    Html,
    Markdown,
    Pod,
}

impl From<PodFmt> for PodFormat {
    fn from(f: PodFmt) -> Self {
        match f {
            PodFmt::Plain => PodFormat::Plain,
            PodFmt::Html => PodFormat::Html,
            PodFmt::Markdown => PodFormat::Markdown,
            PodFmt::Pod => PodFormat::Pod,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Look up a CPAN author by PAUSE id.
    Author {
        /// PAUSE id, e.g. PLICEASE (case-insensitive).
        pauseid: String,
    },

    /// Fetch a distribution release.
    ///
    /// With no --author this is the latest release of the distribution. With
    /// --author, NAME is a full release name including the version, e.g.
    /// FFI-Platypus-2.10.
    Release {
        /// Distribution name, or full release name when --author is given.
        name: String,
        /// PAUSE id of the uploading author.
        #[arg(long)]
        author: Option<String>,
    },

    /// Resolve a module name to the file that provides it.
    Module {
        /// Module name, e.g. FFI::Platypus.
        module: String,
    },

    /// Fetch metadata for one file within a release.
    File {
        /// PAUSE id of the release author.
        author: String,
        /// Full release name, e.g. FFI-Platypus-2.10.
        release: String,
        /// Archive-relative path, e.g. lib/FFI/Platypus.pm.
        path: String,
    },

    /// Print the raw source of one file within a release.
    Source {
        author: String,
        release: String,
        path: String,
    },

    /// Print rendered documentation for a module.
    Pod {
        /// Module name, e.g. FFI::Platypus.
        module: String,
        /// Rendering format.
        #[arg(long, value_enum, default_value = "plain")]
        format: PodFmt,
    },

    /// Fetch distribution-level aggregate data (CPAN River, bugs, ...).
    Distribution {
        /// Distribution name, e.g. FFI-Platypus.
        distribution: String,
    },

    /// Fetch a distribution's change log.
    ///
    /// With no --author this is the change log of the latest release. With
    /// --author, NAME is a full release name including the version.
    Changes {
        /// Distribution name, or full release name when --author is given.
        name: String,
        /// PAUSE id of the release author.
        #[arg(long)]
        author: Option<String>,
    },

    /// Resolve the download URL for the release providing a module.
    DownloadUrl {
        /// Module name, e.g. FFI::Platypus.
        module: String,
        /// Version constraint, e.g. "== 2.08" or "<= 2.10".
        #[arg(long)]
        version: Option<String>,
        /// Allow developer (trial) releases.
        #[arg(long)]
        dev: bool,
    },

    /// List known CPAN mirrors.
    Mirrors,

    /// Search a document type with a Lucene query string.
    Search {
        /// Document type: release, author, module, file, distribution, favorite.
        #[arg(long, short = 't', value_name = "TYPE")]
        r#type: String,
        /// Lucene query, e.g. "author:PLICEASE AND status:latest".
        #[arg(long, short = 'q', value_name = "QUERY")]
        query: String,
        /// Maximum number of hits to return.
        #[arg(long, short = 's')]
        size: Option<u32>,
        /// Offset of the first hit (for pagination).
        #[arg(long)]
        from: Option<u32>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let g = &cli.global;
    let color = match g.color {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => std::io::stdout().is_terminal(),
    };
    let client = build_client(g)?;

    match &cli.command {
        Command::Author { pauseid } => {
            let v = get(&client, &format!("author/{pauseid}")).await?;
            emit(v, g.json, color, |v| render::author(v, color))?;
        }

        Command::Release { name, author } => {
            let v = match author {
                Some(author) => {
                    let v = get(&client, &format!("release/{author}/{name}")).await?;
                    unwrap_key(v, "release")
                }
                None => get(&client, &format!("release/{name}")).await?,
            };
            emit(v, g.json, color, |v| render::release(v, color))?;
        }

        Command::Module { module } => {
            let v = get(&client, &format!("module/{module}")).await?;
            emit(v, g.json, color, |v| render::file_doc(v, color))?;
        }

        Command::File {
            author,
            release,
            path,
        } => {
            let path = path.trim_start_matches('/');
            let v = get(&client, &format!("file/{author}/{release}/{path}")).await?;
            emit(v, g.json, color, |v| render::file_doc(v, color))?;
        }

        Command::Source {
            author,
            release,
            path,
        } => {
            let text = client
                .source(author, release, path.trim_start_matches('/'))
                .await
                .context("fetching source")?;
            emit_text(&text, g.json, color);
        }

        Command::Pod { module, format } => {
            let text = client
                .pod(module, (*format).into())
                .await
                .context("fetching pod")?;
            emit_text(&text, g.json, color);
        }

        Command::Distribution { distribution } => {
            let v = get(&client, &format!("distribution/{distribution}")).await?;
            emit(v, g.json, color, |v| render::distribution(v, color))?;
        }

        Command::Changes { name, author } => {
            let path = match author {
                Some(author) => format!("changes/{author}/{name}"),
                None => format!("changes/{name}"),
            };
            let v = get(&client, &path).await?;
            emit(v, g.json, color, |v| render::changes(v, color))?;
        }

        Command::DownloadUrl {
            module,
            version,
            dev,
        } => {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(version) = version {
                query.push(("version", version.clone()));
            }
            if *dev {
                query.push(("dev", "1".to_string()));
            }
            let v = get_query(&client, &format!("download_url/{module}"), &query).await?;
            emit(v, g.json, color, |v| render::download_url(v, color))?;
        }

        Command::Mirrors => {
            let v = get(&client, "mirror").await?;
            let v = unwrap_key(v, "mirrors");
            emit(v, g.json, color, |v| render::mirrors(v, color))?;
        }

        Command::Search {
            r#type,
            query,
            size,
            from,
        } => {
            let mut params: Vec<(&str, String)> = vec![("q", query.clone())];
            if let Some(size) = size {
                params.push(("size", size.to_string()));
            }
            if let Some(from) = from {
                params.push(("from", from.to_string()));
            }
            let v = get_query(&client, &format!("{type}/_search"), &params).await?;
            if g.json {
                print!("{}", json::to_string(&v, color));
            } else {
                render::search(v, r#type, color)?;
            }
        }
    }

    Ok(())
}

fn build_client(g: &GlobalOpts) -> Result<Client> {
    let mut builder = Client::builder().user_agent(concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION")
    ));
    if let Some(base) = &g.base_url {
        builder = builder.base_url(base.clone());
    }
    if let Some(dir) = &g.cache_dir {
        builder = builder.cache_dir(dir.clone());
    }
    builder.build().context("building HTTP client")
}

/// GET a path and parse the body as JSON.
async fn get(client: &Client, path: &str) -> Result<Value> {
    client
        .get_json::<Value>(path)
        .await
        .with_context(|| format!("GET {path}"))
}

/// GET a path with query parameters and parse the body as JSON. Used for the
/// endpoints the typed crate methods build query strings for (`download_url`,
/// `_search`).
async fn get_query(client: &Client, path: &str, query: &[(&str, String)]) -> Result<Value> {
    let url = client.url(path)?;
    let response = client
        .http()
        .get(url)
        .query(query)
        .send()
        .await
        .with_context(|| format!("GET {path}"))?;
    let status = response.status();
    let body = response.text().await.context("reading response body")?;
    if !status.is_success() {
        anyhow::bail!("MetaCPAN API error {}: {}", status.as_u16(), body.trim());
    }
    serde_json::from_str(&body).with_context(|| format!("parsing {path} response as JSON"))
}

/// Some endpoints wrap their payload in a single-key envelope (`release`,
/// `mirrors`). Return the inner value, or the original if the key is absent.
fn unwrap_key(value: Value, key: &str) -> Value {
    match value {
        Value::Object(mut map) if map.contains_key(key) => map.remove(key).unwrap(),
        other => other,
    }
}

/// Print `value` as JSON, or hand it to `as_table` for the default form.
fn emit(
    value: Value,
    as_json: bool,
    color: bool,
    as_table: impl FnOnce(Value) -> Result<()>,
) -> Result<()> {
    if as_json {
        print!("{}", json::to_string(&value, color));
        Ok(())
    } else {
        as_table(value)
    }
}

/// Content endpoints (`source`, `pod`) return text, not a document. Print it
/// verbatim, or wrap it in `{ "content": ... }` for `--json`.
fn emit_text(text: &str, as_json: bool, color: bool) {
    if as_json {
        print!("{}", json::to_string(&json!({ "content": text }), color));
    } else {
        print!("{text}");
        if !text.ends_with('\n') {
            println!();
        }
    }
}
