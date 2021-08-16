use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "readlater", about, author)]
pub struct Args {
    #[structopt(subcommand)]
    pub cmd: Command,

    #[structopt(short, long)]
    pub verbose: bool,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "newsboat")]
    Newsboat(Newsboat),
    #[structopt(name = "epub")]
    Epub(Epub),
    #[structopt(name = "rss")]
    Rss(Rss),
    #[structopt(name = "cleanup")]
    Cleanup(Cleanup),
    #[structopt(name = "article")]
    Article(Article),
}

#[derive(Debug, StructOpt)]
pub struct Newsboat {
    /// URL of the article
    pub url: String,

    /// Title of the article
    pub title: Option<String>,

    /// Description of the article
    pub desc: Option<String>,

    /// Feed title
    pub feed_title: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct Epub {
    /// Write an epub file to the given file (given locally cached articles)
    #[structopt(name = "filename")]
    pub epub: String,
}

#[derive(Debug, StructOpt)]
pub struct Rss {
    /// Write an rss file to the given file (given locally cached articles)
    #[structopt(name = "filename")]
    pub rss: String,
}

#[derive(Debug, StructOpt)]
pub struct Cleanup {
    /// Cleanup cached articles based on access time.
    #[structopt(name = "seconds")]
    pub age: u8,
}

#[derive(Debug, StructOpt)]
pub struct Article {
    /// Save a readable version of the article
    #[structopt(name = "url")]
    pub url: String,
}
