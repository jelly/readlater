# readlater

## RSS output

```
cargo run -- rss /tmp/test.rss
```

## epub output

Epub generation support is availabe using `pandoc`.

## Newsboat bookmark

```
bookmark-cmd "/home/jelle/projects/readlater/target/debug/readlater newsboat"
bookmark-autopilot yes
```

## TODO

* ammonia for sanitizing html
