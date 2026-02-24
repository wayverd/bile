# TODO

## code stats

might be interesting to try and add github style code stats

look into these libraries

  - https://crates.io/crates/gengo
    - this one actually supports reading from a bare git repo
  - https://crates.io/crates/hyperpolyglot
    - this one does not support bare git repos

## default file handling

other than specific routes all files are handled as 'raw'

should svg be rendered as source code or as an image

should markdown documents be rendered like README routes or should they be handled like all other code

maybe add a way to switch between rendered and 'raw' views

`?raw` query maybe?

## gitweb style project listing

support something like gitwebs project listing config to allow for a custom repo listing layout

might use the current config for it

maybe something like this?

```toml
[[repo]]
name = "bile"
section = "web services"
```

could even have it be part of the repo config

```ini
[bile]
    section = "web services"
```

## html sanitization for code view and readme

looking into using https://crates.io/crates/ammonia for this as there might be an injection possiblity with code and readme views

## use gix instead of git2 and/or caching repo data

see https://codeberg.org/kallisti5/gitore for this
