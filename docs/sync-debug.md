# Why is the download slow

Every sync measures where its time went and writes the result to
`<instance>/.noro/sync-report.json`, with a summary in the launcher log under
`sync_trace`. A real one, a 3357-file modpack sample on a warm CDN:

```
time per file: server wait 51%, transfer 48%, disk 0%, hashing 0%
first byte, HIT: p50 76 ms, p90 138 ms, p99 265 ms over 3353 files
LATENCY-BOUND: most files are small and most of their time is waiting for the first byte ...
sync: 3357 files, 107.3 MB in 19.9 s of download (5.4 MB/s, peak 9.1) over 4 connections, 30/48 slots busy on average, 0 retries; total 19.9 s
```

## Live view

```sh
NORO_SYNC_DEBUG=1 noro-launcher-core      # or set it before starting the launcher
```

The sync screen then shows one line under the progress bar, updated every
second:

```
6.1 MB/s · 112/128 in flight on 4 conn · 2140/3357 files · ttfb p50 117 ms · disk 3% · CDN hit 98% · retries 0
```

and the report keeps every file instead of the 50 slowest.

## Reading it

Times per file overlap, so they are shares, not wall clock: "disk 45%" means
45% of the time files spent being fetched went into waiting for writes.

| Finding | What it means | What helps |
|---|---|---|
| `DISK-BOUND` | writes are slow | on Windows, exclude the launcher folder from the antivirus |
| `LATENCY-BOUND` | small files waiting for the first byte | more in flight (stage concurrency) |
| `SLOT-BOUND` | every slot busy and still latency-bound | raise the stage's concurrency |
| `UNDERUSED` | few slots busy — a tail of large files | large files are already split across connections |
| `CDN COLD` | Cloudflare had to fetch from the origin | a second sync is faster; Tiered Cache / Cache Reserve on the zone |
| `NETWORK-BOUND` | transfer dominates | `NORO_SYNC_CONNECTIONS=8`; if nothing changes, it is the link |

## Knobs

- `NORO_SYNC_CONNECTIONS` (default 4, 1–16) — separate TCP connections the
  sync spreads over. Throughput to the origin is capped per connection, and
  HTTP/2 puts every request of one client on one connection.
- Files of 16 MB and more are fetched as parallel byte ranges over all
  connections and checked against their SHA-1 as a whole; a server that
  ignores ranges gets an ordinary download.

## Comparing settings on a real build

```sh
curl -H "Authorization: Bearer $NORO_ADMIN_TOKEN" \
  "$NORO_MASTER_URL/api/admin/builds/<build_id>/files" > files.json
cargo run --release -p backend --example sync_bench -- \
  files.json https://cdn.example.com/files 1,4,8 2800
BENCH_CONCURRENCY=128 cargo run --release -p backend --example sync_bench -- ...
```

Every run downloads into an empty directory, like a first install, and prints
the same findings. The first run of a build may be a cold CDN; repeat the
first setting at the end to compare like with like.
