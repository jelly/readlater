# readlater

Readlater is a small command line utility designed to convert web articles into
`readable` articles using [readability](https://crates.io/crates/readability).

It integrates into [newsboat](https://newsboat.org/)'s bookmark command or can
be provided with an article url.

## RSS output

```
cargo run -- rss /tmp/test.rss
```

## epub output

Epub generation support is availabe but requires `pandoc` to be installed.

## Newsboat bookmark

```
bookmark-cmd "/home/jelle/projects/readlater/target/debug/readlater newsboat"
bookmark-autopilot yes
```
