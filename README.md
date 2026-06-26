# fcm-rs

Rust library and binary for sending Firebase Cloud Messaging (FCM) push notifications via the FCM v1 REST API. Authenticates using a Google service account key file.

## Features

- FCM v1 API (`/v1/projects/{project}/messages:send`)
- Google service account authentication via `gauth`
- Typed error handling with `thiserror`
- Async-first (`tokio` via `actix-web`)
- Android notification support (title, body, image, sound, click action)

## Setup

### 1. Service account key

Download a service account JSON key from the [Firebase Console](https://console.firebase.google.com/) with the **Firebase Cloud Messaging API** role enabled.

### 2. Environment variable

```sh
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
```

Or add it to a `.env` file in the project root:

```env
GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
```

## Usage

### As a binary

```sh
cargo run --release
```

### As a library

```rust
use fcm_rs::handlers::fcm_handler::FCMHandlerV1;
use fcm_rs::models::fcm_payload::FCMPayloadData;

let handler = FCMHandlerV1::new();

let payload = FCMPayloadData {
    title: "Hello!".to_string(),
    message: "This is a test notification.".to_string(),
    image: "https://example.com/image.png".to_string(),
    click_action: "OPEN_APP".to_string(),
};

handler.push(&payload, "<device-fcm-token>").await?;
```

## Error handling

`push` and `send_notification_to` return `Result<(), FcmError>`. Variants:

| Variant | Cause |
|---|---|
| `ServiceKeyRead` | Key file missing or unreadable |
| `ServiceKeyParse` | Key file is not valid JSON |
| `MissingProjectId` | `project_id` field absent in key |
| `AccessToken` | OAuth2 token fetch failed |
| `HttpRequest` | HTTP transport error |

## Dependencies

| Crate | Purpose |
|---|---|
| `actix-web` | Async runtime |
| `isahc` | HTTP client |
| `gauth` | Google service account OAuth2 |
| `serde_json` | JSON serialization |
| `thiserror` | Typed error enum |
| `dotenv` | `.env` file support |
