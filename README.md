# studipfs

## Instructions

### Prerequisites

You need to install the [rust toolchain](https://www.rust-lang.org/tools/install) for building.

Compiling the fuse crate depends on fuse >= 2.6 (not 3.x).

### Build & Install

Install to your local environment:
```sh
$ cargo install --path .
```

Don't forget to `source "$HOME/.cargo/env"`

### Usage

Set API base URL:
```sh
$ export STUDIP_API_URL="https://studip.example.org/api.php"
```

Set Basic Authorization Token:
```sh
$ export STUDIP_TOKEN=$(echo -n "$username:$password" | base64 | sed -e 's/^/Basic /')
```

At the time of writing, there's no way to directly pass the course id (`cid`) from a URL like
`https://studip.example.org/dispatch.php/course/overview?cid=0123456789abcdef0123456789abcdef`.
However the folder id of a course can be obtained with this shell command (uses `curl` and `jq`):

```sh
$ curl -s -u '$username:$password' "$STUDIP_API_URL/course/$cid/top_folder" | jq -r '.id'
```

Mount and run in background:
```sh
$ studipfs <folder id> <mountpoint> &
```

Unmount:
```sh
$ fusermount -u <mountpoint>
```
