use anyhow::Result;
use chrono::prelude::*;
use platform_dirs::AppDirs;
use readability::extractor;
use select::document::Document;
use select::predicate::Name;
use sqlite::Value;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::process::Command;

pub mod args;

fn get_db_connection() -> Result<sqlite::Connection> {
    let mut app_dirs = AppDirs::new(Some(env!("CARGO_PKG_NAME")), false).unwrap();
    fs::create_dir_all(&app_dirs.data_dir)?;

    app_dirs.data_dir.push("urls.db");

    let connection = sqlite::open(app_dirs.data_dir)?;
    connection
        .execute("CREATE TABLE IF NOT EXISTS articles (url TEXT UNIQUE, title TEXT, html TEXT, description TEXT DEFAULT '', created DATETIME DEFAULT CURRENT_TIMESTAMP);")
        .unwrap();

    Ok(connection)
}

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
    desc: Option<String>,
    _feed_title: Option<String>,
) -> Result<String> {
    let title = match title {
        Some(title) => title,
        None => get_url_title(&url)?,
    };
    let desc = desc.unwrap_or_else(|| String::from(""));

    let connection = get_db_connection()?;
    let data = extractor::scrape(url.as_str())?;

    let mut cursor = connection
        .prepare("INSERT INTO articles (url, title, html, description) VALUES (?, ?, ?, ?)")
        .unwrap()
        .into_cursor();

    cursor
        .bind(&[
            Value::String(url),
            Value::String(title),
            Value::String(data.content),
            Value::String(desc),
        ])
        .unwrap();

    cursor.next()?;

    Ok(String::from("Article saved"))
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
    let connection = get_db_connection()?;
    let mut items = Vec::new();

    let mut cursor = connection
        .prepare("SELECT title,html,description,created FROM articles")
        .unwrap()
        .into_cursor();

    while let Some(row) = cursor.next().unwrap() {
        let articlename = row[0].as_string().unwrap();
        let data = row[1].as_string().unwrap();
        let description = row[2].as_string().unwrap();
        let created = row[3].as_string().unwrap();

        let dt = Local.datetime_from_str(created, "%Y-%m-%d %H:%M:%S")?;

        let guid = rss::GuidBuilder::default()
            .value(articlename)
            .permalink(true)
            .build()
            .map_err(anyhow::Error::msg)?;

        let item = rss::ItemBuilder::default()
            .title(Some(articlename.into()))
            .description(Some(description.into()))
            .content(String::from(data))
            .pub_date(Some(dt.to_rfc2822()))
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
