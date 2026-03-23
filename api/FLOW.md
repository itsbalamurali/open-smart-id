# Smart-ID API Flows

## Overview

The Smart-ID platform has three actors:

- **Relying Party (RP)** — a website or service that wants to authenticate or sign on behalf of a user
- **Smart-ID Server** — this API (`localhost:3000`)
- **Mobile App** — the user's phone running the SmartID Flutter app

All flows follow the same pattern: the RP creates a session, the user confirms on their phone, and the RP polls for the result.

---

## 1. Device-Link Authentication

Used when the RP can show a QR code or deep link to the user.

```
RP                          Server                        App
│                             │                            │
│ POST /v3/authentication/    │                            │
│   device-link/etsi/{id}     │                            │
│ ───────────────────────────>│                            │
│                             │ create session (RUNNING)   │
│ <─── DeviceLinkResponse ───│                            │
│   sessionID                 │                            │
│   sessionToken              │                            │
│   sessionSecret             │                            │
│   deviceLinkBase            │                            │
│                             │                            │
│ show QR code to user        │                            │
│ (deviceLinkBase + token)    │                            │
│                             │                            │
│                             │     user scans QR code     │
│                             │ <─────────────────────────│
│                             │                            │
│                             │  GET /app/v1/sessions/{id} │
│                             │ <─────────────────────────│
│                             │ ── session detail ────────>│
│                             │                            │
│                             │    user reviews & confirms │
│                             │                            │
│                             │  POST /app/v1/sessions/    │
│                             │    {id}/confirm            │
│                             │ <─────────────────────────│
│                             │                            │
│                             │ issue certificate          │
│                             │ complete session (OK)      │
│                             │ ── AppSessionAction ──────>│
│                             │                            │
│ GET /v3/session/{id}        │                            │
│   ?timeoutMs=30000          │                            │
│ ───────────────────────────>│                            │
│                             │ (long poll returns)        │
│ <── SessionStatusResponse ─│                            │
│   state: COMPLETE           │                            │
│   result.endResult: OK      │                            │
│   result.documentNumber     │                            │
│   cert.value (X.509)        │                            │
│   signature (ACSP_V2)       │                            │
```

### Endpoints used

| Actor | Method | Path |
|-------|--------|------|
| RP | POST | `/v3/authentication/device-link/etsi/{id}` |
| RP | POST | `/v3/authentication/device-link/document/{doc}` |
| RP | POST | `/v3/authentication/device-link/anonymous` |
| RP | GET | `/v3/session/{sessionID}?timeoutMs=` |
| App | GET | `/app/v1/sessions/{sessionID}` |
| App | POST | `/app/v1/sessions/{sessionID}/confirm` |
| App | POST | `/app/v1/sessions/{sessionID}/refuse` |

---

## 2. Notification Authentication

Used when the RP knows the user's identity but cannot show a QR code (e.g. cross-device login). A push notification is sent to the user's registered device.

```
RP                          Server                        App
│                             │                            │
│ POST /v3/authentication/    │                            │
│   notification/etsi/{id}    │                            │
│ ───────────────────────────>│                            │
│                             │ create session (RUNNING)   │
│ <── { sessionID } ─────────│                            │
│                             │                            │
│                             │ ── FCM push ──────────────>│
│                             │   { sessionId, kind }      │
│                             │                            │
│ GET /v3/session/{id}        │                            │
│   ?timeoutMs=60000          │    app receives push       │
│ ───────────────────────────>│                            │
│   (long polling...)         │ GET /app/v1/sessions/{id}  │
│                             │ <─────────────────────────│
│                             │ ── session detail ────────>│
│                             │                            │
│                             │    user confirms           │
│                             │                            │
│                             │ POST /app/v1/sessions/     │
│                             │   {id}/confirm             │
│                             │ <─────────────────────────│
│                             │                            │
│                             │ complete session (OK)      │
│ <── SessionStatusResponse ─│ (long poll returns)        │
│   state: COMPLETE           │                            │
```

### FCM push payload

```json
{
  "notification": {
    "title": "SmartID",
    "body": "DEMO requests authentication"
  },
  "data": {
    "sessionId": "uuid",
    "sessionKind": "authentication"
  }
}
```

---

## 3. Signing Flow

Identical structure to authentication, but uses different endpoints and the `RAW_DIGEST_SIGNATURE` protocol instead of `ACSP_V2`.

| Flow | RP endpoint |
|------|-------------|
| Device-link signing | `POST /v3/signature/device-link/etsi/{id}` or `/document/{doc}` |
| Notification signing | `POST /v3/signature/notification/etsi/{id}` or `/document/{doc}` |
| Linked signing | `POST /v3/signature/notification/linked/{doc}` |

Notification signing also returns a **verification code (VC)** — a 4-digit number the RP must display so the user can verify they're signing the correct document.

---

## 4. Certificate Choice Flow

When the RP doesn't know the user's document number, they use certificate choice to let the user pick which Smart-ID account to use.

```
RP                          Server
│                             │
│ POST /v3/signature/         │
│  certificate-choice/        │
│  device-link/anonymous      │
│ ───────────────────────────>│
│ <── DeviceLinkResponse ────│
│                             │
│ (user completes on app)     │
│                             │
│ GET /v3/session/{id}        │
│ ───────────────────────────>│
│ <── documentNumber ────────│
│                             │
│ POST /v3/signature/         │
│  notification/linked/{doc}  │
│ ───────────────────────────>│
│ (signing session follows)   │
```

---

## 5. Certificate Retrieval

Direct certificate lookup when the RP already knows the document number.

```
RP                          Server
│                             │
│ POST /v3/signature/         │
│  certificate/{doc}          │
│ ───────────────────────────>│
│ <── CertificateResponse ───│
│   state: OK                 │
│   cert.value (DER+Base64)   │
│   cert.certificateLevel     │
```

---

## 6. App Device Registration

Before receiving push notifications, the app must register.

```
App                         Server
│                             │
│ POST /app/v1/devices/       │
│   register                  │
│ { semanticId, fcmToken,     │
│   platform }                │
│ ───────────────────────────>│
│                             │ find/create account
│                             │ store device + FCM token
│ <── { deviceId,             │
│       accountId,            │
│       documentNumber } ────│
```

---

## Session States

| State | Meaning |
|-------|---------|
| `RUNNING` | Waiting for user action on the mobile app |
| `COMPLETE` | User has confirmed or refused, or timeout occurred |

## End Results

| Result | Meaning |
|--------|---------|
| `OK` | User confirmed successfully |
| `USER_REFUSED` | User explicitly declined |
| `TIMEOUT` | No response within time limit |
| `DOCUMENT_UNUSABLE` | Account/certificate problem |
| `WRONG_VC` | Wrong verification code selected |
| `USER_REFUSED_CERT_CHOICE` | User cancelled account selection |
| `USER_REFUSED_INTERACTION` | User cancelled during interaction |
| `PROTOCOL_FAILURE` | Signing protocol error |

## Signature Protocols

| Protocol | Used for | Response contains |
|----------|----------|-------------------|
| `ACSP_V2` | Authentication | `serverRandom`, `userChallenge`, `flowType`, signature value |
| `RAW_DIGEST_SIGNATURE` | Signing | `flowType`, signature value |
