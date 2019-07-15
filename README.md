# teapot

A high performance `HTTP/1.1` server that responds `HTTP/1.1 418 I'm a teapot` to GETs

### Run locally

On host

```
cargo run
```

In container

```bash
docker run -p 8080:8080 kirinrastogi/teapot
```

### Dockerstuff

Image [here](https://hub.docker.com/r/kirinrastogi/teapot)
