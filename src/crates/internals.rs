use reqwest::header;
use serde::Deserialize;

use super::error::CratesError;

const USER_AGENT: &str = "Youknow-sys/tgcaptcha-rs";

#[derive(Debug)]
pub enum CrateVariant {
    /// a link to the doc (usually used for rustc crates)
    Link(String),
    Crate(Crate),
}

#[derive(Debug, Deserialize)]
struct Crates {
    crates: Vec<Crate>,
}

#[derive(Debug, Deserialize)]
pub struct Crate {
    pub name: String,
    pub max_version: Option<String>,
    pub max_stable_version: Option<String>,
    pub updated_at: String,
    pub downloads: u64,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub repository: Option<String>,
    exact_match: bool,
}

/// Lookup crates on crates.io
pub async fn crate_lookup(
    http: &reqwest::Client,
    crate_name: &str,
) -> Result<CrateVariant, CratesError> {
    if let Some(url) = rustc_crate_link(crate_name) {
        return Ok(CrateVariant::Link(url.to_owned()));
    }

    get_crate(http, crate_name).await.map(CrateVariant::Crate)
}

/// Lookup documentation
pub async fn doc_lookup(http: &reqwest::Client, query: &str) -> Result<String, CratesError> {
    let mut query_iter = query.splitn(2, "::");

    let Some(first_path_element) = query_iter.next() else {
        return Err(CratesError::InvalidQueryInDocLookup);
    };

    let mut doc_url = if let Some(rustc_crate) = rustc_crate_link(first_path_element) {
        rustc_crate.to_owned()
    } else if first_path_element.is_empty() || is_in_std(first_path_element) {
        "https://doc.rust-lang.org/stable/std/".to_owned()
    } else {
        get_documentation(&get_crate(http, first_path_element).await?)
    };

    if is_in_std(first_path_element) {
        doc_url += "?search=";
        doc_url += &query;
    } else if let Some(item_path) = query_iter.next() {
        doc_url += "?search=";
        doc_url += item_path;
    }

    Ok(doc_url)
}

pub fn get_documentation(crate_info: &Crate) -> String {
    match crate_info.documentation {
        Some(ref doc) => doc.to_owned(),
        None => format!("https://docs.rs/{}", crate_info.name),
    }
}

/// Queries the crates.io crates list for a specific crate
async fn get_crate(http: &reqwest::Client, query: &str) -> Result<Crate, CratesError> {
    log::info!("searching for crate `{}`", query);

    let crate_list = http
        .get("https://crates.io/api/v1/crates")
        .header(header::USER_AGENT, USER_AGENT)
        .query(&[("q", query)])
        .send()
        .await?
        .json::<Crates>()
        .await
        .map_err(CratesError::CratesIoParseJson)?;

    let Some(crate_info) = crate_list.crates.into_iter().next() else {
        return Err(CratesError::CrateNotFound(None));
        // bail!("Crate <code>{query}</code> not found")
    };

    if crate_info.exact_match {
        Ok(crate_info)
    } else {
        Err(CratesError::CrateNotFound(Some(crate_info.name)))
        // bail!(
        //     "Crate <code>{query}</code> not found. Did you mean <code>{}</code>?",
        //     crate_info.name
        // )
    }
}

/// todo
#[allow(unused)]
pub async fn matching_crates(http: &reqwest::Client, partial: &str) -> impl Iterator<Item = Crate> {
    let response = http
        .get("https://crates.io/api/v1/crates")
        .header(header::USER_AGENT, USER_AGENT)
        .query(&[("q", partial), ("per_page", "25"), ("sort", "downloads")])
        .send()
        .await;

    let crate_list = match response {
        Ok(response) => response.json::<Crates>().await.ok(),
        Err(_) => None,
    };

    crate_list
        .map_or(Vec::new(), |list| list.crates)
        .into_iter()
}

/// Returns whether the given type name is the one of a primitive.
fn is_in_std(name: &str) -> bool {
    name.chars().next().map(char::is_uppercase).unwrap_or(false)
        || [
            "f32",
            "f64",
            "i8",
            "i16",
            "i32",
            "i64",
            "i128",
            "isize",
            "u8",
            "u16",
            "u32",
            "u64",
            "u128",
            "usize",
            "char",
            "str",
            "pointer",
            "reference",
            "fn",
            "bool",
            "slice",
            "tuple",
            "unit",
            "array",
        ]
        .contains(&name)
}

/// Provide the documentation link to an official Rust crate (e.g. std, alloc, nightly)
fn rustc_crate_link(crate_name: &str) -> Option<&'static str> {
    match crate_name.to_ascii_lowercase().as_str() {
        "std" => Some("https://doc.rust-lang.org/stable/std"),
        "core" => Some("https://doc.rust-lang.org/stable/core"),
        "alloc" => Some("https://doc.rust-lang.org/stable/alloc/"),
        "proc_macro" | "proc-macro" => Some("https://doc.rust-lang.org/stable/proc_macro"),
        "beta" => Some("https://doc.rust-lang.org/beta/std"),
        "nightly" => Some("https://doc.rust-lang.org/nightly/std"),
        "rustc" => Some("https://doc.rust-lang.org/nightly/nightly-rustc"),
        "test" => Some("https://doc.rust-lang.org/stable/test"),
        _ => None,
    }
}
