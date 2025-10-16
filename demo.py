#!/usr/bin/env python
import time
import json

from websockets.sync.server import serve


def schedule(websocket):
    websocket.send("[]")
    i = 0
    for message in websocket:
        print(message)
        packets = [
            {
                "start": None,
                "lifetime": 3600,
                "draw": {"Rectangle": {"width": 2000, "height": 5}},
                "x": {"position": 0, "pixels": 0, "frames": 1},
                "y": {"position": 32, "pixels": 1, "frames": 1},
            },
            {
                "start": None,
                "lifetime": 60,
                "draw": "Dvd",
                "x": {"position": 160, "pixels": 5, "frames": 1},
                "y": {"position": 1000, "pixels": 5, "frames": 1},
            },
        ]
        packets.append(
            {
                "start": None,
                "lifetime": 3600,
                "draw": "Milo",
                "x": {"position": 160, "pixels": 5, "frames": 1},
                "y": {"position": 320, "pixels": 0, "frames": 1},
            }
        )
        websocket.send(json.dumps(packets))
        time.sleep(1)


def main():
    with serve(schedule, "0.0.0.0", 9001) as server:
        server.serve_forever()


if __name__ == "__main__":
    main()
