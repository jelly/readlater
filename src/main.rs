use readlater::args::{Args, Command};
use structopt::StructOpt;

pub fn main() {
    let args = Args::from_args();

    match args.cmd {
        Command::Newsboat(cmd) => {
            match readlater::readable_article(cmd.url, cmd.title, cmd.desc, cmd.feed_title) {
                Ok(output) => {
                    if args.verbose {
                        println!("{}", output);
                    }
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Unable to generate epub");
                    eprintln!("Reason: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Epub(cmd) => match readlater::generate_epub(&cmd.epub) {
            Ok(output) => {
                if args.verbose {
                    println!("{}", output);
                }
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Unable to generate epub");
                eprintln!("Reason: {}", e);
                std::process::exit(1);
            }
        },
        Command::Rss(cmd) => match readlater::generate_rss(&cmd.rss) {
            Ok(output) => {
                if args.verbose {
                    println!("{}", output);
                }
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Unable to generate rss feed");
                eprintln!("Reason: {}", e);
                std::process::exit(1);
            }
        },
        Command::Cleanup(_c) => {}
        Command::Article(cmd) => match readlater::readable_article(cmd.url, None, None, None) {
            Ok(output) => {
                if args.verbose {
                    println!("{}", output);
                }
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Unable to generate html");
                eprintln!("Reason: {}", e);
                std::process::exit(1);
            }
        },
    }
}
