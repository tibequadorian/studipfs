# studipfs

## Instructions

### Build & Install

Build:
```
$ cargo build
```

and install:

```
$ cargo install --path .
```

Don't forget to `source "$HOME/.cargo/env"`

### Usage

Set API base URL:
```
$ export STUDIP_API_URL=https://studip.example.org/api.php"
```

Set Basic Authorization Token:
```
$ export STUDIP_TOKEN=$(echo -n "$username:$password" | base64 | sed -e 's/^/Basic /')
```

Mount
```
$ studipfs <folder id> <mountpoint> &
```

Unmount
```
$ fusermount -u <mountpoint>
```
