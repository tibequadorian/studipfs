# studipfs

## About

`studipfs` is a filesystem client to mount any Stud.IP directory.

It currently provides basic functionality while keeping a very small code base. PRs are very welcome!

## Instructions

### Prerequisites

The [rust toolchain](https://www.rust-lang.org/tools/install) is required for building.

Compilation of `fuser` crate has additional [dependencies](https://crates.io/crates/fuser#dependencies).

### Build & Install

Install to local environment:
```sh
$ cargo install --path .
```

### Usage

**CAUTION: Storing login data in enviroment variables is extremely insecure! Use at your own risk.**

Set API base URL:
```sh
$ export STUDIP_API_URL="https://studip.example.org/api.php"
```

Set Basic Authorization Token:
```sh
$ export STUDIP_TOKEN=$(echo -n "$username:$password" | base64 | sed -e 's/^/Basic /')
```

At the time of writing, there's no way to directly pass the course id (`cid`) from a URL like in
`https://studip.example.org/dispatch.php/course/overview?cid=6675636b206361706974616c69736d21`.
However the folder id of a course can be obtained with this shell command (using `curl` and `jq`):

```sh
$ curl -s -u "$username:$password" "$STUDIP_API_URL/course/$cid/top_folder" | jq -r '.id'
```

Mount and run in background:
```sh
$ studipfs <folder id> <mountpoint> &
```

Unmount:
```sh
$ fusermount -u <mountpoint>
```
