# `bile`

a simple self-hosted git server.

its a fork of [thebotlynoob/agit](https://github.com/TheBotlyNoob/agit) (which
is a fork of [alexwennerberg/mygit](https://github.com/alexwennerberg/mygit)),
along with some styling based of [stagit](https://git.codemadness.org/stagit/).

for a live example see my instance: https://git.wayver.dev.

**NOTE**: this is alpha software, it wont break anything, but i am and will be
making large changes to both the code and the config, so be sure that check
that you don't need to change anything before updating

## download

as bile is in alpha, there are currently no ways to download a built version

in the future there will be both binary and docker downloads

## building

youll need the rust compiler and a c compiler

run `cargo build --release`

the resulting binary is in `target/release`

## setup

### server setup

get a copy of the server

move it where ever you wish to run it from

by default it looks for a config file in the same directory

if it doesnt find one it'll use its default values (see `src/config.rs`)

highly recommend making your own config (see
[server configuration](#server-configuration))

create a folder for the repos `mkdir repos` (or what ever you set that config
to)

run `bile` (either manually, via your init system, or something like docker)

and place it behind a reverse proxy

### repo setup

create a bare repo with `git init --bar <repo_name>`

enter the newly made bare repo `cd <repo_name>`

tell bile that its allowed to show the repo `touch git-daemon-export-ok`

rename the post-update hook (see hook for details)
`mv hooks/post-update.sample hooks/post-update`

update `HEAD` to point to the main branch (normally master or main), for
example `echo "ref: refs/heads/main" > HEAD`

update the `description` file so show a proper description on the web page

set optional git config flags (see [git configuration](#git-configuration))
with `git config <flag_name> <flag_value>`

pushing to the repo is handled by ssh, thats left as an exercise to the reader

## configuration

### server configuration

heres an example config showing all the configuration options

```toml
# the port the server will listen on
port = 5000
# Directory to find git repos
project_root = "./repos"
# the text shown in a browsers title bar
site_name = "wayver's git archive"
# file to check for in the .git directory to decide whether to publicly show a repo
export_ok = "git-daemon-export-ok"
# base URL to clone repositories from (without trailing slash)
clone_base = "https://git.wayver.dev"
# the number of commits to be shown when paginating the log
log_per_page = 100
```

### git configuration

currently only supports 2 custom values

  - `gitweb.owner`: sets repo owner, currently only used for rss feeds
  - `bile.section`: sets the (visual) section of the repo on the home page

## faq
