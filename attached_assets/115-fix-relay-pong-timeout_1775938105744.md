# Issue #115 — Fix Relay Pong Timeout Disconnects

## Review Assessment

**Verdict: Legit.** The diagnosis is correct and the fix is clean.

Replit's HTTP proxy terminates the upstream WebSocket and opens a new one to your server, which means RFC 6455 control frames (opcode `0x9`/`0xA` ping/pong) are consumed by the proxy and never forwarded. Your relay's `relayLastPong` therefore never updates, the 60-90 s timeout fires, and the server kills a perfectly healthy connection. The node daemon reconnects, the cycle repeats, and the logs fill with "no pong" entries on ~5-minute intervals.

## Files Required from Replit

Export a ZIP containing **exactly** these files:

| # | Path | Why |
|---|------|-----|
| 1 | `server/index.ts` | Primary fix target — ping interval handler (~L2682-2737), pong listener (~L2758-2764), authenticated-message handler (~L2926-2929), and app-level ping handler (~L3174-3176) |
| 2 | `server/services/node-watchdog.ts` | Context only — confirm watchdog thresholds (L22-25) are untouched by this change |

No other files are in scope.

## Patch Plan (4 edits, 1 file)

All edits are in `server/index.ts`. The watchdog file is read-only context.

### Edit 1 — Belt-and-suspenders: reset `relayLastPong` on every authenticated inbound message

**Location:** ~L2926-2928, where `entry.lastSeen` is already updated on any inbound message.

**Change:** After `entry.lastSeen = Date.now();`, add `entry.relayLastPong = Date.now();`

**Rationale:** Any traffic from the node proves the connection is alive. This is the broadest fix and makes Edits 2-3 redundant as defense-in-depth, but we do all three for clarity.

### Edit 2 — Application-level `{ type: "ping" }` also resets `relayLastPong`

**Location:** ~L3174-3176, inside the `if (parsed.type === "ping")` handler.

**Change:** Add `entry.relayLastPong = Date.now();` alongside any existing `entry.lastSeen` update.

**Rationale:** Makes the intent explicit — an app-level heartbeat satisfies the pong check. Readable and self-documenting even though Edit 1 already covers it.

### Edit 3 — Pong timeout check also considers `entry.lastSeen`

**Location:** ~L2682-2737, inside the `setInterval` ping handler where `relayLastPong` is compared against the timeout threshold.

**Change:** Replace the condition:

```ts
// BEFORE
if (now - entry.relayLastPong > PONG_TIMEOUT) {
```

with:

```ts
// AFTER
const lastActivity = Math.max(entry.relayLastPong, entry.lastSeen);
if (now - lastActivity > PONG_TIMEOUT) {
```

**Rationale:** Even if Edits 1-2 are somehow bypassed (race condition, future refactor), the timeout check itself now respects any recent activity. A node is only killed when it has sent *zero* traffic of any kind for the full timeout window.

### Edit 4 — No change needed to the protocol-level pong listener

**Location:** ~L2758-2764.

**Status:** Leave as-is. It still updates `relayLastPong` for environments where control frames *do* arrive (local dev, non-Replit). No regression.

## Verification

After deployment, confirm:

1. `grep "no pong" logs` returns zero hits over a 10-minute window.
2. Cluster-health endpoint shows all three nodes (`111.111.111.111.1`, `211.111.111.111.1`, `311.111.111.111.1`) with stable `connectedSince` timestamps that do not reset.
3. Kill a node daemon process — the server should detect the dead connection within the existing timeout window and prune it. (Proves no regression in dead-connection detection.)

## Out of Scope (confirmed)

- Client-side daemon reconnection logic — untouched.
- Node-watchdog thresholds (UP/SUSPECT/DOWN) — untouched.
- LauncherPanel WebSocket client — untouched.
