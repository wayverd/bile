# `bile`

a simple self-hosted git server.

its a fork of [thebotlynoob/agit](https://github.com/TheBotlyNoob/agit) (which is a fork of [alexwennerberg/mygit](https://github.com/alexwennerberg/mygit)),
along with some styling based of [stagit](https://git.codemadness.org/stagit/).

## config

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
