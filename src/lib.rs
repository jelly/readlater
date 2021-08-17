use anyhow::Result;
use readability::extractor;
use select::document::Document;
use select::predicate::Name;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

pub mod args;

fn get_default_cachedir() -> String {
    let homedir = match env::var("HOME") {
        Ok(val) => val,
        // TODO: should never happen...
        Err(_) => "/tmp/".to_string(),
    };

    match env::var("XDG_CACHE_HOME") {
        // TODO: avoid hardcoding
        Ok(val) => format!("{}/readlater", val),
        Err(_) => format!("{}/.cache/readlater", homedir),
    }
}

fn get_url_title(url: &str) -> Result<String> {
    let resp = reqwest::blocking::get(url)?;
    let doc = Document::from_read(resp)?;
    let elem = doc.find(Name("title")).next();

    let title = match elem {
        Some(elem) => elem.text(),
        None => String::from("Untitled"),
    };

    Ok(String::from(title.trim()))
}

pub fn readable_article(
    url: String,
    title: Option<String>,
    _desc: Option<String>,
    _feed_title: Option<String>,
) -> Result<String> {
    let title = match title {
        Some(title) => title,
        None => get_url_title(&url)?,
    };

    let cache_dir = get_default_cachedir();
    fs::create_dir_all(&cache_dir)?;
    let filename = format!("{}/{}.html", cache_dir, title);

    let data = extractor::scrape(url.as_str())?;
    let mut file = File::create(&filename)?;
    write!(file, "{}", data.content)?;

    Ok(format!("{} {}", "Article saved to", filename))
}

pub fn generate_epub(epub: &str) -> Result<String> {
    let cache_dir = get_default_cachedir();
    let mut pandoc = Command::new("pandoc");
    pandoc
        .arg("--metadata")
        .arg("title='readlater'")
        .arg("--toc")
        .arg("--toc-depth=1")
        .arg("--standalone")
        .arg("--output")
        .arg(&epub);

    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let extension = path.extension().unwrap_or_else(|| std::ffi::OsStr::new(""));
        if extension != "html" {
            continue;
        }

        let filepath = path.into_os_string().into_string().unwrap();
        pandoc.arg(filepath);
    }

    let output = pandoc.output()?;

    Ok(String::from_utf8(output.stdout)?)
}

pub fn generate_rss(rss: &str) -> Result<String> {
    let cache_dir = get_default_cachedir();
    let mut items = Vec::new();

    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        let duppath = path.clone();
        if path.is_dir() {
            continue;
        }

        let extension = path.extension().unwrap_or_else(|| std::ffi::OsStr::new(""));
        if extension != "html" {
            continue;
        }

        let filepath = path.into_os_string().into_string().unwrap();
        let epubfilename = duppath.file_name().unwrap().to_str().unwrap();
        let articlename = epubfilename.strip_suffix(".html").unwrap();

        let data = std::fs::read_to_string(&filepath)?;

        let guid = rss::GuidBuilder::default()
            .value(articlename)
            .permalink(true)
            .build()
            .map_err(anyhow::Error::msg)?;

        let item = rss::ItemBuilder::default()
            .title(Some(articlename.into()))
            .description(Some(articlename.into()))
            .content(Some(data))
            .guid(guid)
            .build()
            .map_err(anyhow::Error::msg)?;
        items.push(item);
    }

    // TODO: configuration
    let channel = rss::ChannelBuilder::default()
        .title("Web Articles")
        .link("https://dodgy.download/articles.rss")
        .description("desc")
        .items(items)
        .build()
        .map_err(anyhow::Error::msg)?;

    let rss_string = channel.to_string();
    let mut rss_file = fs::File::create(&rss)?;
    rss_file.write_all(&rss_string.into_bytes())?;
    rss_file.write_all(b"\n")?;

    Ok(format!("RSS file written to {}", &rss))
}
