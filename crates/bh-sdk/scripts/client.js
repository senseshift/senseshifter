// npm i ws
// node client.js "ws://127.0.0.1:15881/v4/feedback?workspace_id=<workspace_id>&api_key=<api_key>&version=<version>"

const crypto = require("crypto");
const WebSocket = require("ws");

// ---- config ----
const WS_URL = process.argv[2];

const log = (...a) => console.log("[client]", ...a);
const b64ToBuf = (b64) => Buffer.from(b64, "base64");
const bufToB64 = (buf) => Buffer.from(buf).toString("base64");
const b64urlToB64 = (s) => s.replace(/-/g, "+").replace(/_/g, "/");

// Wrap a base64 SPKI into PEM
function spkiB64ToPem(b64) {
    const body = (b64.match(/.{1,64}/g) || [b64]).join("\n");
    return `-----BEGIN PUBLIC KEY-----\n${body}\n-----END PUBLIC KEY-----\n`;
}

// AES-GCM utils (12-byte IV prefix, tag at the end)
function encryptSdkDataB64(aesKeyBuf, jsonString) {
    const iv = crypto.randomBytes(12);
    const cipher = crypto.createCipheriv("aes-256-gcm", aesKeyBuf, iv);
    const ct = Buffer.concat([cipher.update(jsonString, "utf8"), cipher.final()]);
    const tag = cipher.getAuthTag();
    return bufToB64(Buffer.concat([iv, ct, tag]));
}
function decryptSdkDataB64(aesKeyBuf, dataB64) {
    const raw = b64ToBuf(dataB64);
    if (raw.length < 12 + 16) throw new Error("cipher too short");
    const iv = raw.slice(0, 12);
    const ctt = raw.slice(12, -16);
    const tag = raw.slice(-16);
    const dec = crypto.createDecipheriv("aes-256-gcm", aesKeyBuf, iv);
    dec.setAuthTag(tag);
    const pt = Buffer.concat([dec.update(ctt), dec.final()]);
    return pt.toString("utf8");
}

function connect() {
    log("connecting to", WS_URL);
    const ws = new WebSocket(WS_URL);

    // Will be filled after ServerKey + SdkClientKey handshake
    let aesKey = null;
    let serverPem = null;

    function sendJson(obj) {
        const s = JSON.stringify(obj);
        ws.send(s);
        log("->", s.length > 200 ? s.slice(0, 200) + "…" : s);
    }

    function sendSdkData(obj) {
        if (!aesKey) {
            log("cannot send SdkData yet: AES key not established");
            return;
        }
        const plaintext = JSON.stringify(obj);
        const dataB64 = encryptSdkDataB64(aesKey, plaintext);
        sendJson({ Type: "SdkData", Key: null, Data: dataB64 });
    }

    ws.on("open", () => log("WebSocket OPEN"));

    ws.on("message", (raw) => {
        const t = raw.toString("utf8");
        let msg;
        try { msg = JSON.parse(t); } catch { log("<- non-JSON", t.slice(0, 200)); return; }

        // Log compactly
        log("<-", msg.Type || "?", t.length > 200 ? t.slice(0, 200) + "…" : t);

        // 1) ServerKey -> build PEM, generate AES, send SdkClientKey
        if (msg.Type === "ServerKey" && msg.Key) {
            const keyB64 = /^[A-Za-z0-9+/=]+$/.test(msg.Key) ? msg.Key : b64urlToB64(msg.Key);
            serverPem = spkiB64ToPem(keyB64);
            log("ServerKey looks like SPKI, length:", keyB64.length);

            aesKey = crypto.randomBytes(32);
            const enc = crypto.publicEncrypt(
                { key: serverPem, padding: crypto.constants.RSA_PKCS1_PADDING },
                aesKey
            );
            const encB64 = bufToB64(enc);

            sendJson({ Type: "SdkClientKey", Key: encB64, Data: null });
            log("sent SdkClientKey (RSA PKCS#1 v1.5), aes =", aesKey.toString("hex"));

            // Optional: immediately ping after establishing AES
            setTimeout(() => {
                sendSdkData({ Type: "SdkPingAll", Message: "" });
            }, 50);

            return;
        }

        // 2) Encrypted data from server
        if (msg.Type === "SdkData" && msg.Data) {
            if (!aesKey) {
                log("received SdkData before AES established");
                return;
            }
            try {
                const plaintext = decryptSdkDataB64(aesKey, String(msg.Data));
                log("SdkData <-", plaintext);

                // If payload is JSON, you can branch on it here:
                try {
                    const obj = JSON.parse(plaintext);
                    if (obj?.Type === "SdkPongAll") {
                        log("got pong:", obj);
                    }
                    // handle more message types as needed…
                } catch {}
            } catch (e) {
                log("AES decrypt error:", e.message);
            }
            return;
        }

        // Other messages (plain)
        // You can inspect and optionally respond here
    });

    ws.on("close", (code, reason) => log("CLOSE", code, reason?.toString?.() || ""));
    ws.on("error", (err) => log("ERROR", err.message));

    // Expose a tiny REPL-ish API in case you want to send things interactively:
    process.on("SIGINT", () => {
        log("closing…");
        ws.close();
        process.exit(0);
    });

    return {
        sendSdkData: (obj) => sendSdkData(obj),
    };
}

connect();
