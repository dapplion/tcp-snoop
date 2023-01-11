# tcp-snooper

Log to stdout all data streamed through TCP connections

## Install

Install and run with cargo

```
cargo install tcp-snooper
tcp-snooper --help
```

Pull and run with docker

```
docker run ghcr.io/dapplion/tcp-snooper --help
```

## Usage

_terminal 1_: Target

```
python3 -m http.server 8000
```

_terminal 2_: TCP snooper pointing to target at `127.0.0.1:8000` and listening at port `5000`

```
tcp-snooper 127.0.0.1:5000 127.0.0.1:8000
```

_terminal 3_: Consumer connects to snooper at `127.0.0.1:5000` instead of original target at `127.0.0.1:8000`

```
curl 127.0.0.1:5000
```

Can print data as UTF8 by default or as a hex with `--encoding hex`
