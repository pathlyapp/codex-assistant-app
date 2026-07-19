# Router Contract

Codex Assistant configures an OpenAI-compatible Router. Payment and device activation are not part of this contract.

## Base URL

The configured URL must use HTTP or HTTPS and end in `/v1`:

```text
https://router.example.com/v1
```

Production deployments should use HTTPS. Plain HTTP is intended for localhost and private-network development.

## Authentication

If authentication is enabled, Codex Assistant sends:

```http
Authorization: Bearer <user-supplied-key>
```

On Windows the key is protected with current-user DPAPI. The Codex config invokes the installed Rust token helper; it does not contain the key.

## Required endpoints

### `GET /v1/models`

```json
{
  "object": "list",
  "data": [
    {
      "id": "qwen3.5:122b-a10b",
      "object": "model",
      "owned_by": "router"
    }
  ]
}
```

Requirements:

- Return HTTP 2xx.
- Return at least one non-empty `data[].id`.
- Return 401/403 for an invalid key.
- Keep model IDs stable after configuration.

### `POST /v1/responses`

The Router must implement the OpenAI Responses-compatible protocol used by Codex, including streaming and tool calls required by the selected model. Codex Assistant writes:

```toml
wire_api = "responses"
```

The installer currently validates `/models`; Windows end-to-end acceptance must additionally run a real Codex request against `/responses`.

## Operational requirements

- Request IDs in response headers.
- Structured audit logs without prompts or bearer tokens by default.
- Explicit timeouts and stable error JSON.
- Health monitoring and model availability tracking.
- Key rotation and revocation for commercial deployment.
- No upstream provider secret should be distributed to clients when the Router can hold it server-side.
