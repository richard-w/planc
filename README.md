# planc

**Simple Planning Poker application**

---

This is just a simple multiplayer Planning Poker application. I wrote this because holding cards
into the camera while working from home does not work well.

## Goals

* Enable efficient participation in planning poker sessions while working from home.
* Enable people in meeting rooms to participate via their mobile phones.
* Provide some basic statistics about the results of each round.

## Setup

### Docker

The easiest way to setup the application. Just build the image and run it.

```bash
docker build -t planc .
docker run --rm -it \
    -p 8080:8080 \
    -e MAX_SESSIONS=1 \
    -e MAX_USERS=8 \
    planc
```

You can now open the application in your webbrowser (localhost:8080).

### Development

Direct build via cargo requires that the `trunk` utility is in `$PATH`. You can install it via
```bash
cargo install trunk
```
