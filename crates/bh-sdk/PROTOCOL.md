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
3. I discoveded this project, also in Rust, that seems to be emulate a bH WebSocket server: `https://github.com/VRC-Haptics/VRCH-Server/blob/main/src-tauri/src/bhaptics/game/v3/mod.rs`, though v2 is empty.
4. Found this C# MIT project that implements the protocol: `https://github.com/HerpDerpinstine/bHapticsLib`
5. Also these: https://github.com/V1perN3st/bHaptics-Firebot/blob/main/src/bHaptics/bHapticsPlayer.ts, 