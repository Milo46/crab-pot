# Role-Based Access Control (RBAC)

## Problem

When an API key is compromised, the attacker has access to the entire functionality
of the software. Users who are responsibl  solely for uploading logs to a schema
should never be able to delete schemas. The goal is to limit blast-radius by granting
each key only the minimum permissions it actually needs.

## Solution

Assign every API key a **role**. Each role maps to a fixed set of **permissions**.
Every HTTP handler declares which permission it requires; requests that do not satisfy
that requirement are rejected with `403 Forbidden` before reaching handler logic.

---

## Step 1 — Define permissions

Permissions are granular, single-action capabilities scoped to a resource:

| Permission       | Description                                  |
|------------------|----------------------------------------------|
| `schemas_read`   | `GET` schemas (list, get by ID/name)         |
| `schemas_write`  | `POST` / `PUT` schemas (create, update)      |
| `schemas_delete` | `DELETE` schemas                             |
| `logs_read`      | `GET` logs (list, get by ID, cursors)        |
| `logs_write`     | `POST` logs (ingest)                         |
| `logs_delete`    | `DELETE` logs                                |
| `ws_connect`     | Connect to the `/ws/logs` WebSocket endpoint |

---

## Step 2 — Define roles

Roles are named bundles of permissions assigned to a key at creation time.
The default role for a new key is `writer`.

| Role             | Permissions granted                                                              |
|------------------|----------------------------------------------------------------------------------|
| `reader`         | `schemas_read`, `logs_read`, `ws_connect`                                        |
| `writer`         | `schemas_read`, `logs_read`, `logs_write`, `ws_connect`                          |
| `schema_manager` | `schemas_read`, `schemas_write`, `schemas_delete`, `logs_read`                   |
| `log_manager`    | `schemas_read`, `logs_read`, `logs_write`, `logs_delete`, `ws_connect`           |
| `admin`          | All permissions                                                                  |

**Typical key assignments:**

- Log producer service → `writer`
- Monitoring dashboard → `reader`
- Schema deployment pipeline → `schema_manager`
- Log archival/cleanup job → `log_manager`
- Internal tooling / admin use → `admin`

---

## Step 3 — Assign a role to an API key

Pass `role` when creating a key via the admin API. If omitted, `writer` is used.

```http
POST /admin/v1/api-keys
Content-Type: application/json

{
  "name": "sensor-ingest",
  "role": "writer"
}
```

---

## Step 4 — Enforce permissions per route

Each route group is wrapped with a `require_permission` middleware layer that reads
the `Role` from the injected `ApiKey` extension and checks it against the required
`Permission`. Authentication (key lookup) always runs first.

```
Request
  └─ api_key_middleware        → authenticates, injects Arc<ApiKey>
       └─ require_permission   → checks key's role has the required permission
            └─ handler         → executes only if authorized
```

Unauthorized requests receive `403 Forbidden` with no further detail exposed.

---

## Step 5 — Inspect your own key

Any authenticated key can call `GET /v1/me` to discover its own role and the full
list of permissions it holds — no extra database query required.

```http
GET /v1/me
Authorization: Bearer sk_abc123...
```

```json
{
  "key_prefix": "sk_abc123...",
  "name": "sensor-ingest",
  "role": "writer",
  "permissions": ["schemas_read", "logs_read", "logs_write", "ws_connect"]
}
```

This endpoint is useful for onboarding integrators, debugging unexpected `403` responses,
and allowing client SDKs to gate features based on available permissions.
