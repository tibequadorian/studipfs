# studipfs

## Instructions

How to obtain Basic Authorization Token:
```
$ echo -n "$username:$password" | base64 | sed -e 's/^/Basic /'
```
