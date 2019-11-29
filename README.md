# hyper-chunky

This webserver responds with an outrageous `transfer-encoding: chunked`
stream which seems to confuse epoll in some situations.

http://localhost:4432/29/3 will return 29 chunks of 3 bytes each.

Hyper, as locked, writes these as:
```
strace -f -ewritev target/release/hyper-chunky
...
[pid 17378] writev(40, [{iov_base="3\r\n", iov_len=3}, {iov_base="qqq", iov_len=3}, {iov_base="\r\n", iov_len=2}, ...
2019-11-29T22:02:21.366949624 (+PT0.000098041S) hyper::proto::h1::io flushed 109 bytes
```

... which you could imagine being stressful, but also how `writev` is
supposed to be used.
