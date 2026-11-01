# Paperdash Image Backend

The backend is a Rust/Rocket HTTP service. It captures the configured Paperdash
UI in headless Chromium, caches the PNG per device, converts it into the
black/white and red bit planes expected by the four-panel e-paper display, and
serves each panel section as a raw binary response.

## Requirements

- Rust and Cargo (edition 2021).
- Chromium/Chrome executable accessible to the service process.
- A reachable Paperdash UI instance.

## Configuration

The service reads `./config.json` relative to its working directory at startup.
The file must contain `website_url` and `devices`:

```json
{
  "website_url": "http://127.0.0.1:8080",
  "devices": [
    {
      "id": "*",
      "widgets": [
        {
          "id": "text",
          "options": [{ "id": "text", "value": "# Hello\nDashboard is online." }]
        }
      ]
    }
  ]
}
```

Device IDs are matched case-insensitively. An exact ID takes precedence; `*`
is used when no exact match exists. Widget options use `{ "id", "value" }`
entries, and `value` is a string. Supported widget IDs and their options are
documented in [`../paperdash-ui/README.md`](../paperdash-ui/README.md).
The repository's [`config.json.example`](config.json.example) uses widget IDs
and option names accepted by the current UI; copy it to `config.json` and edit
the device IDs and values for your installation. In Docker, use the UI service
name and container port, `http://paperdash-ui:80`, so Chromium reaches the UI
over the private Docker network rather than going out through the public host.

The firmware identifies itself with its Wi-Fi MAC address, formatted as
colon-separated hexadecimal. Set `website_url` to a UI address reachable from
the backend host/container. The firmware's backend host must be the address
reachable from the ESP32, not necessarily the backend's container address.

Set `CHROME` to the Chromium/Chrome executable path when it cannot be detected
automatically. The Dockerfile sets `CHROME=/usr/bin/chromium`, listens on port
80 inside its container, and installs Chromium. When running locally, Rocket
defaults to port 8000; Rocket's standard environment variables, such as
`ROCKET_PORT` and `ROCKET_ADDRESS`, can override its bind settings.
For a locally run backend that must accept requests from an ESP32 on the LAN,
bind to all interfaces, for example:

```sh
ROCKET_ADDRESS=0.0.0.0 cargo run
```

Allow the selected port through the host firewall. Binding to `0.0.0.0` does
not add authentication or encryption; see the security notes below.

## Production HTTPS and Docker

Terminate HTTPS at a reverse proxy (for example, Caddy, Traefik, or Nginx) with
certificates for `paperdash.b15g.de` and `paperdash-backend.b15g.de`. Route the
UI host to `127.0.0.1:8080` and the backend host to `127.0.0.1:8000`. Keep the
backend's Rocket listener on plain HTTP behind the proxy; the ESP32 connects to
`https://paperdash-backend.b15g.de` on port 443, while the backend uses
`http://paperdash-ui:80` internally for screenshots. Do not publish the
containers' port 80 directly to the internet.

The Compose file builds both images, places them on a private network, binds
their host ports to loopback for the reverse proxy, and mounts your device
configuration read-only. Create/edit `config.json` first, then run from this
directory:

```sh
cp config.json.example config.json
docker compose up --build -d
```

Point the firmware to `paperdash-backend.b15g.de`, port `443`, with TLS enabled
(the defaults in `prj.conf`). Ensure both DNS records point to the reverse
proxy and that the proxy can access the loopback ports. The backend loads
configuration once at startup, so restart it after changing the file. The
GitHub Actions workflow builds and publishes the image to
`ghcr.io/<owner>/<repository>` when changes are pushed to `main`.

## Run Locally

From this directory, with Chromium installed and `config.json` configured:

```sh
cargo run
```

To run the optimized binary:

```sh
cargo build --release --locked
CHROME=/path/to/chromium ./target/release/paperdash-backend
```

Run from the directory containing `config.json`; the current implementation
loads that relative path at startup. The backend does not expose an API for
editing/reloading configuration while running.

## HTTP API

All routes are mounted under `/paperdash`. Each requires the `id` query
parameter and returns an unframed raw bitmap body (not PNG, JSON, or a
base64-encoded image):

| Route | Region | Width | Body size |
| --- | --- | ---: | ---: |
| `/paperdash/s2/bw?id=<device>` | top-left | 648 px | 39,852 bytes |
| `/paperdash/m2/bw?id=<device>` | top-right | 656 px | 40,344 bytes |
| `/paperdash/m1/bw?id=<device>` | bottom-left | 648 px | 39,852 bytes |
| `/paperdash/s1/bw?id=<device>` | bottom-right | 656 px | 40,344 bytes |
| `/paperdash/s2/r?id=<device>` | top-left | 648 px | 39,852 bytes |
| `/paperdash/m2/r?id=<device>` | top-right | 656 px | 40,344 bytes |
| `/paperdash/m1/r?id=<device>` | bottom-left | 648 px | 39,852 bytes |
| `/paperdash/s1/r?id=<device>` | bottom-right | 656 px | 40,344 bytes |

Each region is 492 pixels tall. Rows are packed most-significant bit first,
eight pixels per byte: `81 * 492` bytes for narrow panels and `82 * 492` for
wide panels. The black/white plane is dithered to monochrome; the red plane
marks pixels whose RGB values meet the red threshold (`r >= 128`, `g < 128`,
`b < 128`). The firmware applies the controller-specific red-plane inversion.

Example request:

```sh
curl --fail --output /tmp/s2-bw.bin \
  'http://127.0.0.1:8000/paperdash/s2/bw?id=00:11:22:33:44:55'
wc -c /tmp/s2-bw.bin
```

## Screenshot Cache and Failure Behavior

The cache is in memory and keyed by device ID. A screenshot is generated on the
first request and refreshed after 60 seconds. Screenshot capture uses Chromium
at exactly 1304 x 984 pixels; an unexpected screenshot size or browser error is
logged and is not cached. Image route handlers currently return white fallback
bytes if the cache is absent, the PNG cannot be decoded, or a requested region
is outside the screenshot bounds. Check backend logs and route payloads if a
display region is blank.

## Security Notes

The backend's internal image routes use plain HTTP and have no authentication;
the reverse proxy provides public HTTPS transport but does not add
authentication. Anyone who can reach the backend hostname can request device
images, so apply suitable proxy rate/access controls if that is a concern.
Chromium is launched with its sandbox and browser web-security checks disabled
for screenshot compatibility. Only point it at a UI instance and widget content
you trust.

## Tests and Source Map

```sh
cargo fmt --check
cargo test --locked
cargo build --release --locked
```

- `src/main.rs`: routes, cache lifetime, screenshot-to-bitmap conversion, and tests.
- `src/modules/config.rs`: JSON configuration and device/widget selection.
- `src/modules/screenshot.rs`: Chromium setup, exact viewport, and PNG capture.
- `Dockerfile`: release image and Chromium runtime dependency.
