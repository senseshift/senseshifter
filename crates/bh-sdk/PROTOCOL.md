# bH Protocol Observations

## Specification

- Uses WebSocket communication in port `15881`.

### v2 Protocol

## Tools used
- [`examples/ws-logging-server`](./examples/ws-logging-server) - A simple WebSocket server that logs incoming messages to the console.

## Observations Log

### 2025-08-27

1.  I've found this GitHub repository under Apache-2.0 license: https://github.com/cercata/pysim2bhap/blob/main/sim2bhap/haptic_player.py.
    Here I can see it uses `ws://localhost:15881/v2/feedbacks` URL to send haptic feedbacks. I made a simple WebSocket server to log incoming messages.
2.  Upon launching, it submitted a ton of data:
    ```
    2025-08-27T05:07:12.115231Z  INFO ws_logging_server: New WebSocket connection: 127.0.0.1:50369
    2025-08-27T05:07:12.115670Z  INFO ws_logging_server: Received close message from 127.0.0.1:50365
    2025-08-27T05:07:12.124956Z  INFO ws_logging_server: Received a message from 127.0.0.1:50369: {"Register": [{"Key": "msfs_vace", "Project": {"Tracks": [{"effects": [{"modes": {"VestBack": {"dotMode": {"dotConnected": false, "fee
    dback": [{"endTime": 41, "playbackType": "NONE", "pointList": [{"index": 8, "intensity": 1}, {"index": 9, "intensity": 1}, {"index": 10, "intensity": 1}, {"index": 11, "intensity": 1}], "startTime": 0}, {"endTime": 83, "playback
    Type": "NONE", "pointList": [{"index": 4, "intensity": 1}, {"index": 5, "intensity": 1}, {"index": 6, "intensity": 1}, {"index": 7, "intensity": 1}, {"index": 12, "intensity": 1}, {"index": 13, "intensity": 1}, {"index": 14, "in
    tensity": 1}, {"index": 15, "intensity": 1}], "startTime": 41}, {"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 0, "intensity": 1}, {"index": 1, "intensity": 1}, {"index": 2, "intensity": 1}, {"index": 3, "inten
    sity": 1}, {"index": 16, "intensity": 1}, {"index": 17, "intensity": 1}, {"index": 18, "intensity": 1}, {"index": 19, "intensity": 1}], "startTime": 83}]}, "mode": "DOT_MODE", "pathMode": {"feedback": [{"movingPattern": "CONST_S
    PEED", "playbackType": "NONE", "visible": true, "pointList": []}]}}, "VestFront": {"dotMode": {"dotConnected": false, "feedback": [{"endTime": 41, "playbackType": "NONE", "pointList": [{"index": 8, "intensity": 1}, {"index": 9,
    "intensity": 1}, {"index": 10, "intensity": 1}, {"index": 11, "intensity": 1}], "startTime": 0}, {"endTime": 83, "playbackType": "NONE", "pointList": [{"index": 4, "intensity": 1}, {"index": 5, "intensity": 1}, {"index": 6, "int
    ensity": 1}, {"index": 7, "intensity": 1}, {"index": 12, "intensity": 1}, {"index": 13, "intensity": 1}, {"index": 14, "intensity": 1}, {"index": 15, "intensity": 1}], "startTime": 41}, {"endTime": 125, "playbackType": "NONE", "
    pointList": [{"index": 0, "intensity": 1}, {"index": 1, "intensity": 1}, {"index": 2, "intensity": 1}, {"index": 3, "intensity": 1}, {"index": 16, "intensity": 1}, {"index": 17, "intensity": 1}, {"index": 18, "intensity": 1}, {"
    index": 19, "intensity": 1}], "startTime": 83}]}, "mode": "DOT_MODE", "pathMode": {"feedback": [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true, "pointList": []}]}}}, "name": "Effect 1", "offsetTime": 12
    5, "startTime": 0}], "enable": true}, {"enable": true, "effects": []}], "Layout": {"layouts": {"VestBack": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "
    y": 0}, {"index": 4, "x": 0, "y": 0.25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10
    , "x": 0.667, "y": 0.5}, {"index": 11, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y":
    1}, {"index": 17, "x": 0.333, "y": 1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}], "VestFront": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index":
    3, "x": 1, "y": 0}, {"index": 4, "x": 0, "y": 0.25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5},
    {"index": 10, "x": 0.667, "y": 0.5}, {"index": 11, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y": 1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}]}, "name": "Tactot", "type": "Tactot"}}}]}
    2025-08-27T05:07:12.133437Z  INFO ws_logging_server: Received a message from 127.0.0.1:50369: {"Register": [{"Key": "msfs_vaoa", "Project": {"Tracks": [{"effects": [{"modes": {"VestBack": {"dotMode": {"dotConnected": true, "feed
    back": [{"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 12, "intensity": 1}, {"index": 13, "intensity": 1}, {"index": 14, "intensity": 1}, {"index": 15, "intensity": 1}], "startTime": 0}, {"endTime": 125, "playb
    ackType": "NONE", "pointList": [{"index": 16, "intensity": 1}, {"index": 17, "intensity": 1}, {"index": 18, "intensity": 1}, {"index": 19, "intensity": 1}], "startTime": 125}]}, "mode": "DOT_MODE", "pathMode": {"feedback": [{"mo
    vingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true, "pointList": []}]}}, "VestFront": {"dotMode": {"dotConnected": false, "feedback": [{"endTime": 125, "playbackType": "NONE", "startTime": 0, "pointList": []}]}
    , "mode": "PATH_MODE", "pathMode": {"feedback": [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true, "pointList": []}]}}}, "name": "Effect 1", "offsetTime": 125, "startTime": 0}], "enable": true}, {"enable"
    : true, "effects": []}], "Layout": {"layouts": {"VestBack": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"index": 4, "x": 0, "y": 0.25}, {"inde
    x": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.667, "y": 0.5}, {"index": 11, "x": 1,
    "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y": 1}, {"index
    ": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}], "VestFront": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"index": 4, "x": 0, "y":
    0.25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.667, "y": 0.5}, {"index":
    11, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y": 1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}]}, "name": "Tactot", "type": "Tactot"}}}]}
    2025-08-27T05:07:12.141454Z  INFO ws_logging_server: Received a message from 127.0.0.1:50369: {"Register": [{"Key": "msfs_vvne", "Project": {"Tracks": [{"effects": [{"modes": {"VestBack": {"dotMode": {"dotConnected": true, "feed
    back": [{"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 0, "intensity": 1}, {"index": 1, "intensity": 1}, {"index": 2, "intensity": 1}, {"index": 3, "intensity": 1}, {"index": 11, "intensity": 1}, {"index": 10,
    "intensity": 1}, {"index": 9, "intensity": 1}, {"index": 8, "intensity": 1}], "startTime": 0}, {"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 4, "intensity": 1}, {"index": 5, "intensity": 1}, {"index": 6, "inte
    nsity": 1}, {"index": 7, "intensity": 1}], "startTime": 125}]}, "mode": "DOT_MODE", "pathMode": {"feedback": [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true, "pointList": []}]}}, "VestFront": {"dotMode"
    : {"dotConnected": false, "feedback": [{"endTime": 125, "playbackType": "NONE", "startTime": 0, "pointList": []}]}, "mode": "DOT_MODE", "pathMode": {"feedback": []}}}, "name": "Effect 1", "offsetTime": 125, "startTime": 0}], "en
    able": true}, {"enable": true, "effects": []}], "Layout": {"layouts": {"VestBack": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"index": 4, "x"
    : 0, "y": 0.25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.667, "y": 0.5},
    {"index": 11, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x":
    0.333, "y": 1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}], "VestFront": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"i
    ndex": 4, "x": 0, "y": 0.25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.66
    7, "y": 0.5}, {"index": 11, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y": 1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}]}, "name": "Tactot", "type": "Tactot"}}}]}
    2025-08-27T05:07:12.149040Z  INFO ws_logging_server: Received a message from 127.0.0.1:50369: {"Register": [{"Key": "msfs_vrpm", "Project": {"Tracks": [{"effects": [{"modes": {"VestBack": {"dotMode": {"dotConnected": false, "fee
    dback": [{"endTime": 125, "playbackType": "NONE", "startTime": 0, "pointList": []}]}, "mode": "PATH_MODE", "pathMode": {"feedback": [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true, "pointList": []}]}},
    "VestFront": {"dotMode": {"dotConnected": false, "feedback": [{"endTime": 62, "playbackType": "NONE", "pointList": [{"index": 0, "intensity": 1}, {"index": 1, "intensity": 1}, {"index": 2, "intensity": 1}, {"index": 3, "intensit
    y": 1}, {"index": 8, "intensity": 1}, {"index": 9, "intensity": 1}, {"index": 10, "intensity": 1}, {"index": 11, "intensity": 1}], "startTime": 0}, {"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 4, "intensity":
    1}, {"index": 5, "intensity": 1}, {"index": 6, "intensity": 1}, {"index": 7, "intensity": 1}], "startTime": 62}]}, "mode": "DOT_MODE", "pathMode": {"feedback": [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible"
    : true, "pointList": []}]}}}, "name": "Effect 1", "offsetTime": 125, "startTime": 0}], "enable": true}, {"enable": true, "effects": []}], "Layout": {"layouts": {"VestBack": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333,
    "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"index": 4, "x": 0, "y": 0.25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "
    x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.667, "y": 0.5}, {"index": 11, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0
    .75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y": 1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}], "VestFront": [{"index": 0, "x": 0, "y": 0}, {"index": 1
    , "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"index": 4, "x": 0, "y": 0.25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {
    "index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.667, "y": 0.5}, {"index": 11, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y": 1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}]}, "name": "Tactot", "type": "Tactot"}}}]}  
    2025-08-27T05:07:12.156192Z  INFO ws_logging_server: Received a message from 127.0.0.1:50369: {"Register": [{"Key": "msfs_vgfe", "Project": {"Tracks": [{"effects": [{"modes": {"VestBack": {"dotMode": {"dotConnected": false, "fee
    dback": [{"endTime": 125, "playbackType": "NONE", "startTime": 0, "pointList": []}]}, "mode": "PATH_MODE", "pathMode": {"feedback": [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true, "pointList": []}]}},
    "VestFront": {"dotMode": {"dotConnected": false, "feedback": [{"endTime": 62, "playbackType": "NONE", "pointList": [{"index": 12, "intensity": 1}, {"index": 13, "intensity": 1}, {"index": 14, "intensity": 1}, {"index": 15, "inte
    nsity": 1}], "startTime": 0}, {"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 16, "intensity": 1}, {"index": 17, "intensity": 1}, {"index": 18, "intensity": 1}, {"index": 19, "intensity": 1}], "startTime": 62}]}
    , "mode": "DOT_MODE", "pathMode": {"feedback": [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true, "pointList": []}]}}}, "name": "Effect 1", "offsetTime": 125, "startTime": 0}], "enable": true}, {"enable":
    true, "effects": []}], "Layout": {"layouts": {"VestBack": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"index": 4, "x": 0, "y": 0.25}, {"index
    ": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.667, "y": 0.5}, {"index": 11, "x": 1, "
    y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y": 1}, {"index"
    : 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}], "VestFront": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"index": 4, "x": 0, "y": 0
    .25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.667, "y": 0.5}, {"index":
    11, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y": 1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}]}, "name": "Tactot", "type": "Tactot"}}}]}
    2025-08-27T05:07:12.163608Z  INFO ws_logging_server: Received a message from 127.0.0.1:50369: {"Register": [{"Key": "msfs_arpm", "Project": {"Tracks": [{"effects": [{"modes": {"ForearmL": {"dotMode": {"dotConnected": true, "feed
    back": [{"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 0, "intensity": 1}, {"index": 1, "intensity": 1}, {"index": 2, "intensity": 1}], "startTime": 0}, {"endTime": 125, "playbackType": "NONE", "pointList": [{"
    index": 3, "intensity": 1}, {"index": 4, "intensity": 1}, {"index": 5, "intensity": 1}], "startTime": 125}]}, "mode": "DOT_MODE", "pathMode": {"feedback": [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true
    , "pointList": []}]}}, "ForearmR": {"dotMode": {"dotConnected": false, "feedback": [{"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 0, "intensity": 1}, {"index": 1, "intensity": 1}, {"index": 2, "intensity": 1}]
    , "startTime": 0}, {"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 3, "intensity": 1}, {"index": 4, "intensity": 1}, {"index": 5, "intensity": 1}], "startTime": 125}]}, "mode": "DOT_MODE", "pathMode": {"feedback
    ": [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true, "pointList": []}]}}}, "name": "Effect 1", "offsetTime": 125, "startTime": 0}], "enable": true}, {"enable": true, "effects": []}], "Layout": {"layouts"
    : {"ForearmL": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.5, "y": 0}, {"index": 2, "x": 1, "y": 0}, {"index": 3, "x": 0, "y": 1}, {"index": 4, "x": 0.5, "y": 1}, {"index": 5, "x": 1, "y": 1}], "ForearmR": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.5, "y": 0}, {"index": 2, "x": 1, "y": 0}, {"index": 3, "x": 0, "y": 1}, {"index": 4, "x": 0.5, "y": 1}, {"index": 5, "x": 1, "y": 1}]}, "name": "Tactosy2", "type": "Tactosy2"}}}]}
    2025-08-27T05:07:12.171406Z  INFO ws_logging_server: Received a message from 127.0.0.1:50369: {"Register": [{"Key": "msfs_vfla", "Project": {"Tracks": [{"effects": [{"modes": {"VestBack": {"dotMode": {"dotConnected": false, "fee
    dback": [{"endTime": 41, "playbackType": "NONE", "pointList": [{"index": 8, "intensity": 1}, {"index": 11, "intensity": 1}], "startTime": 0}, {"endTime": 83, "playbackType": "NONE", "pointList": [{"index": 4, "intensity": 1}, {"
    index": 7, "intensity": 1}], "startTime": 41}, {"endTime": 125, "playbackType": "NONE", "pointList": [{"index": 0, "intensity": 1}, {"index": 3, "intensity": 1}], "startTime": 83}]}, "mode": "DOT_MODE", "pathMode": {"feedback":
    [{"movingPattern": "CONST_SPEED", "playbackType": "NONE", "visible": true, "pointList": []}]}}, "VestFront": {"dotMode": {"dotConnected": false, "feedback": [{"endTime": 41, "playbackType": "NONE", "pointList": [{"index": 8, "in
    tensity": 1}, {"index": 11, "intensity": 1}], "startTime": 0}, {"endTime": 83, "playbackType": "NONE", "pointList": [{"index": 4, "intensity": 1}, {"index": 7, "intensity": 1}], "startTime": 41}, {"endTime": 125, "playbackType":
    "NONE", "pointList": [{"index": 0, "intensity": 1}, {"index": 3, "intensity": 1}], "startTime": 83}]}, "mode": "DOT_MODE", "pathMode": {"feedback": []}}}, "name": "Effect 1", "offsetTime": 125, "startTime": 0}], "enable": true}
    , {"enable": true, "effects": []}], "Layout": {"layouts": {"VestBack": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"index": 4, "x": 0, "y": 0.
    25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.667, "y": 0.5}, {"index": 1
    1, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y":
    1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}], "VestFront": [{"index": 0, "x": 0, "y": 0}, {"index": 1, "x": 0.333, "y": 0}, {"index": 2, "x": 0.667, "y": 0}, {"index": 3, "x": 1, "y": 0}, {"index": 4, "x
    ": 0, "y": 0.25}, {"index": 5, "x": 0.333, "y": 0.25}, {"index": 6, "x": 0.667, "y": 0.25}, {"index": 7, "x": 1, "y": 0.25}, {"index": 8, "x": 0, "y": 0.5}, {"index": 9, "x": 0.333, "y": 0.5}, {"index": 10, "x": 0.667, "y": 0.5}
    , {"index": 11, "x": 1, "y": 0.5}, {"index": 12, "x": 0, "y": 0.75}, {"index": 13, "x": 0.333, "y": 0.75}, {"index": 14, "x": 0.667, "y": 0.75}, {"index": 15, "x": 1, "y": 0.75}, {"index": 16, "x": 0, "y": 1}, {"index": 17, "x": 0.333, "y": 1}, {"index": 18, "x": 0.667, "y": 1}, {"index": 19, "x": 1, "y": 1}]}, "name": "Tactot", "type": "Tactot"}}}]}
    2025-08-27T05:09:41.296392Z  INFO ws_logging_server: Received close message from 127.0.0.1:50369
    ```
    I'll start developing Rust Structs to represent this protocol.
3.  I discoveded this project, also in Rust, that seems to be emulate a bH WebSocket server: `https://github.com/VRC-Haptics/VRCH-Server/blob/main/src-tauri/src/bhaptics/game/v3/mod.rs`, though v2 is empty.
4.  Found this C# MIT project that implements the protocol: `https://github.com/HerpDerpinstine/bHapticsLib`
5.  Also these: https://github.com/V1perN3st/bHaptics-Firebot/blob/main/src/bHaptics/bHapticsPlayer.ts, 
6.  After implementing some essential structs, game now connects, but effects are not played. 
    It appears that the game sends preliminary `Register` messages to save possible haptic effects in advance, and then sends `Submit` messages to actually play them.
    And for it to be played, the server must respond with `RegisteredKeys` message.
7.  In the headers, games send their name and id (`?app_id={tld.company.game}&app_name={game}`), so I added a logger for that.

### 2025-08-28

1.   After implementing nice architecture for handlers, I can see that V4 API is encrypted:
     ```
     {"Type":"SdkData","Key":null,"Data":"lJtj/qkDllKkE/VuU3yjQHRRNPNUWbXIJyfsjJKDZ9j7iouETuB/j3dt6bN3OxLVGy+JFICe5YgjZP5q5Fmu14uoO2kCucVgD38OitHyHLnLoL8mwcAK4QGOwwjXC/9yzYZscpLApwUOnilBfJH5SpZuDSzGK4nhCwtg=="}
     ```
2.   In the VRC-Haptics I see that:
     1. It fetches some definitions from `https://sdk-apis.bhaptics.com/api/v1/haptic-definitions/workspace-v3/latest?latest-version=-1&api-key=0jTPyQjylL9KOPMoekos&app-id=`
     2. Some files `auth.json` and `raw-response.json`
3.  Since my game uses v4 api, I created a MITM WS proxy to inspect the traffic. Here are their messages:
    ```
    server: {"Type":"ServerKey","Key":"<partially-redacted>GYnbmOA8KWfjSlVndrWb6qH9nxpt6rJUCrkcG2O4ETJP4Q4YDp+58xJFTU9t6gNrtsULX7rn2Jjf69Z1FVzaGflvwyL44eXFvmzHx4S7j7dptPlwrQIDAQAB","Data":null}
    client: {"Type":"SdkClientKey","Key":"AY6N0hPATalXl/<partially-redacted>/mo76+9LXqGa9VYu5sCqBVDnshkk53ATrtgpDkBwXhDBmIWGsDzjMLoLS9DnokXGFkXecsyc72BuMYhQcT46BcGFlcAaMxudkhZqyHK68ovXVlwHZamCfzee2bx1FA==","Data":null}
    server: {"Type":"SdkData","Key":null,"Data":"UczN3kpg8B1PFqtEKam014TiZp4lWfewG1xrFRwOo+Fr0ySDxCfWVKBo+93MWngsRXqVs7DQUJj5EScy1p6cPFP4Pf0j4DHdMKFrYjwnMpWMC1A7xrvaNQXni99WYH/hfbU5LAkzdSZF0loZrkcrmA8SZ/pPRwSZeBb6ofdmx8xrzR0DNnBw4YlwNcoOeEa92C0Zfo7bu5zuAUzcxYYqH88zvAGl8e7jswzQzbCyfv1z0SJG5AGZr41f8TSa/x4iBh6sVIXMFh3E0c2JJCGXimcbRtns/N3S+/zddAsnN/4LMDLlmod4J4i0MISPXlecEp0AUQK5v5fhMIezH/hI2/wXSlaZLpT+XbQlYSwTJ2bSxzdeIhfvQSa1bGKdc3TOocv6q0DOeXPf5gs0EJU8tXHd7DuJWwoIU6r6PcC9bhZKogapqBk1L2bTZjE="}
    client: {"Type":"SdkData","Key":null,"Data":"o0NftnrrE/NJqeckAfJ0dkDc6ca9YeMjR37kjoweX3jHEyINoMUE0KYa/U4VYK6jAEZzOvKUOne/QfUuDTU="}
    client: {"Type":"SdkData","Key":null,"Data":"JfPw+p76Gv79AZMiH6L1XLlaVOBaf9WuEpl9Ai4uyVblxF/q7AhCg/msPN2cjxTrDVdGsFO8eg9RD7DBRlM="}
    client: {"Type":"SdkData","Key":null,"Data":"p49wOGqKpdn+vD+aeQRP/2oSXawGH130MR/vKYlWqzlTQQ8n/ifgm76YpQ1nJX7R+fW/Y8juA77l1yL/oHk="}
    client: {"Type":"SdkData","Key":null,"Data":"/vOtzVvtk6+nL9gBtPe7o/WLRcAk/xrfSmpzsr1xRYrGnuLcri8nxFrSuwQzhQBFTuISBz/<partially-redacted>/pHjJUYw4COLl0YWZWXLT8cRYXI2WVxLR3AV7FVGxjRzp5kR9PwwuhRqn4CLJ3aXgeLuMe4BRswxEE2OauslvyrgvfTdiNKz2xlZYhoLGH2ok20as2a2N5XQ/3H4kLtO6S2HhsgUZp/A=="}
    ```
    
4.  After some trial and error I implemented the JS WebSocket server (in JS since it's faster to test), and it works! [./scripts/server.js](./scripts/server.js)
5.  I also made a simple client to test it: [./scripts/client.js](./scripts/client.js)