// node --security-revert=CVE-2023-46809 server.js
const http   = require("http");
const crypto = require("crypto");
const url    = require("url");
const WebSocket = require("ws");

// ---- RSA keypair (server) ----
const { publicKey, privateKey } = crypto.generateKeyPairSync("rsa", {
    modulusLength: 2048,
    publicKeyEncoding:  { type: "spki",  format: "pem" },
    privateKeyEncoding: { type: "pkcs8", format: "pem" }
});
const spkiDerB64 = publicKey
    .replace(/-----BEGIN PUBLIC KEY-----|-----END PUBLIC KEY-----|\s+/g, "");

// ---- helpers ----
const b64ToBuf = b64 => Buffer.from(b64, "base64");
const b64urlToBuf = s => Buffer.from(s.replace(/-/g, "+").replace(/_/g, "/"), "base64");
const bufToB64 = buf => Buffer.from(buf).toString("base64");

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
    const iv  = raw.slice(0, 12);
    const ctt = raw.slice(12, -16);
    const tag = raw.slice(-16);
    const dec = crypto.createDecipheriv("aes-256-gcm", aesKeyBuf, iv);
    dec.setAuthTag(tag);
    const pt = Buffer.concat([dec.update(ctt), dec.final()]);
    return pt.toString("utf8");
}

// PKCS#1 v1.5 decrypt for SdkClientKey (with base64/base64url tolerance)
function decryptClientKeyPkcs1v15(b64) {
    const candidates = [b64ToBuf(b64), b64urlToBuf(b64)];
    for (const buf of candidates) {
        try {
            return crypto.privateDecrypt(
                { key: privateKey, padding: crypto.constants.RSA_PKCS1_PADDING },
                buf
            );
        } catch {}
    }
    throw new Error("RSA v1.5 decrypt failed (try --security-revert or node-forge)");
}

// ---- HTTP + WS ----
const server = http.createServer();
const wss = new WebSocket.Server({ noServer: true });

server.on("upgrade", (req, socket, head) => {
    const { pathname, query } = url.parse(req.url, true);
    if (pathname === "/v4/feedback") {
        wss.handleUpgrade(req, socket, head, ws => {
            ws._params = query;
            wss.emit("connection", ws, req);
        });
    } else {
        socket.destroy();
    }
});

wss.on("connection", ws => {
    const connId = Math.random().toString(36).slice(2, 8);
    console.log(`[${connId}] client params:`, ws._params);

    // 1) Send ServerKey (SPKI)
    ws.send(JSON.stringify({ Type: "ServerKey", Key: spkiDerB64, Data: null }));

    ws.on("message", raw => {
        let msg;
        try { msg = JSON.parse(raw.toString("utf8")); } catch { return; }

        if (msg.Type === "SdkClientKey" && msg.Key) {
            console.log(`[${connId}] SdkClientKey base64 length:`, msg.Key.length);
            try {
                const aesKey = decryptClientKeyPkcs1v15(String(msg.Key));
                if (aesKey.length !== 32) {
                    console.warn(`[${connId}] Decrypted key len=${aesKey.length} (expected 32)`);
                }
                ws._aes = aesKey;
                console.log(`[${connId}] AES key: ${aesKey.toString("hex")}`);

                // optional welcome
                const welcome = JSON.stringify({ Type: "SdkServerHello", Message: "hi" });
                ws.send(JSON.stringify({ Type: "SdkData", Key: null, Data: encryptSdkDataB64(ws._aes, welcome) }));
            } catch (e) {
                console.error(`[${connId}] SdkClientKey decrypt failed: ${e.message}`);
            }
            return;
        }

        if (msg.Type === "SdkData" && msg.Data) {
            if (!ws._aes) {
                console.warn(`[${connId}] SdkData before AES established`);
                return;
            }
            try {
                const plaintext = decryptSdkDataB64(ws._aes, String(msg.Data));
                console.log(`[${connId}] SdkData <- ${plaintext}`);
                // simple auto-reply to ping
                try {
                    const parsed = JSON.parse(plaintext);
                    if (parsed?.Type === "SdkPingAll") {
                        const reply = JSON.stringify({ Type: "SdkPongAll", ts: Date.now() });
                        ws.send(JSON.stringify({ Type: "SdkData", Key: null, Data: encryptSdkDataB64(ws._aes, reply) }));
                    }
                } catch {}
            } catch (e) {
                console.error(`[${connId}] AES decrypt error: ${e.message}`);
            }
            return;
        }

        // log everything else
        console.log(`[${connId}] other <-`, msg);
    });
});

const PORT = 15881;
server.listen(PORT, "127.0.0.1", () => {
    console.log("\n[mock] SPKI (base64) sent to client:\n", spkiDerB64);
    console.log(`mock server listening on ws://127.0.0.1:${PORT}/v4/feedback`);
});