# Single-Ledger Rehash Cost: Back-of-the-Envelope Analysis

## Executive summary

A global hash chain is computationally feasible for a chat of ordinary size. The BLAKE3 work itself is tiny. The cost to measure is rewriting SQLite rows and dirtying database pages when a new message sorts before the current tail.

For a late insertion at canonical position `p` in a chat containing `N` messages, a straightforward implementation rehashes and updates about:

```text
k = N - p
```

rows. Appends cost almost nothing beyond the normal insert. A message sorting near the beginning of a large history touches almost the whole chat. A batch of recovered messages should be inserted and rehashed once, not rehashed once per message.

**Recommendation:** don't reject old-but-valid history just to avoid rehashing. Benchmark representative on-disk SQLite workloads first. The current chosen direction is per-author chains, so this report is an option comparison, not a reason to switch it back to a single ledger.

## 1. Codebase facts that affect cost

- The database has one serialized `DbWriter` thread. Long rehash transactions delay other writes queued to that writer.
- Database connections use SQLite WAL mode for read-write connections.
- Current `messages` rows contain a row id, topic id, author endpoint, text content, and timestamp. A single-ledger design would add message id, signature, sequence, and chain hash.
- The proposed canonical key is `(sent_at, endpoint_id, message_id)`. Late-arriving old messages can sort before already stored rows.
- The read/UI path already exposes messages from SQLite; a chain rewrite should be transactional so readers see either the old or new committed state.

## 2. Work model

Let:

- `N` = total messages in this chat before the insert;
- `p` = zero-based canonical insertion position;
- `k = N - p` = rows from the insertion point through the prior tail.

An incremental implementation does approximately:

1. Insert the new message.
2. Determine its canonical position.
3. Recompute `chain_hash` for the new row and every following row.
4. Update each affected row's `seq` and `chain_hash` in one transaction.

The main work is $O(k)$ row reads and updates. Hashing is $O(k)$ too, but each hash input is only around 64 bytes (32-byte previous hash + 32-byte message id).

| Case | Approximate affected rows |
|---|---:|
| New message sorts at tail | 1 |
| New message sorts in last 10% | 0.1N |
| New message sorts near middle | 0.5N |
| New message sorts at beginning | N |

If `r` late messages are applied individually, work is roughly `sum(k_i)`. If they are received as one sync batch and rehashed once from the earliest affected position, work is roughly the suffix after that earliest position. Batching can therefore change the cost by a factor approaching `r` for a burst of old messages.

## 3. Hashing cost is not the bottleneck

At 100,000 affected rows, hashing 64 bytes per row means about 6.4 MB of hash input. At 1,000,000 rows it is about 64 MB. Those sizes are modest for BLAKE3 on a modern device. The implementation should still avoid unnecessary serialization and allocations, but disk/page work and DB-thread occupancy are more important than the cryptographic hash throughput.

## 4. Approximate SQLite page work

This is an order-of-magnitude model, not a benchmark. Assume:

- SQLite page size: 4 KiB (common default; verify the actual database setting);
- stored message row including message text and proposed metadata: roughly 350–650 bytes before B-tree/page overhead, varying greatly with content size;
- therefore roughly 5–10 rows per leaf page as a rough range.

If an insertion at the beginning updates all rows, the table's leaf pages are all dirtied. Approximate WAL page volume for one full-tail rewrite:

| Chat messages | Rows updated | Approx. leaf pages touched | Approx. page bytes written once to WAL* |
|---:|---:|---:|---:|
| 10,000 | 10,000 | 1,000–2,000 | 4–8 MB |
| 100,000 | 100,000 | 10,000–20,000 | 40–80 MB |
| 1,000,000 | 1,000,000 | 100,000–200,000 | 400–800 MB |

\*Illustrative upper-level estimate for a pass that modifies most leaf pages, not a prediction of exact WAL size or latency. SQLite cache spills, indexes, checkpoints, row overflow pages, row layout, and rowid access order can move the result substantially. The WAL may also retain frames until checkpointing; readers can delay checkpoint completion.

The canonical index scan orders by timestamp/author/message id, while the rows are physically organized by local row id. Depending on correlation between arrival and canonical order, table-page lookups during rehash may be fairly sequential or scattered. That is another reason to benchmark rather than infer latency from row count alone.

If a full 100k-message rewrite produces roughly 40–80 MB of page writes, raw storage bandwidth suggests a lower bound ranging from tens to hundreds of milliseconds on ordinary SSD/mobile storage, before SQL statement overhead, page-cache misses, transaction commit, and UI write contention. On slower flash or under contention it can take longer. This is a rough bound, not a measured result.

## 5. Expected workload matters more than worst-case complexity

### Normal local sends

If the device clock is reasonably aligned and messages arrive in timestamp order, most local messages append near the tail. `k` is small and the rehash is cheap.

### Gossip from multiple authors

Concurrent sends and clock skew can cause an insert into the middle. One isolated message in a 100k-message chat can trigger a large rewrite. Repeated out-of-order inserts arriving one at a time can be worse than one batch.

### Synchronization

Sync routinely delivers older messages, so a late insertion is not an edge case for the sync path. A global ledger implementation must batch missing messages and rehash once per batch. It must also make retries idempotent so retransmission doesn't cause pointless work.

### Malicious timestamps

Because the author chooses `sent_at`, a member can intentionally place a message near the start and force a large rewrite. An age-window check can limit how far back new timestamps may sort, but it does not place a strict bound on rows rewritten unless message rate/count is also bounded.

## 6. Why a timestamp cutoff is not a free optimization

Rejecting messages older than, for example, the last hour seems to bound rehash depth, but creates a data-consistency rule:

- A peer offline for longer than the window may be unable to recover valid messages authored during that period.
- Clock skew may make legitimate messages appear outside the allowed window.
- Different peers can observe different latest timestamps and apply the cutoff differently unless the reference watermark is part of shared protocol state.
- A time window bounds duration, not volume. A very active chat can still have many rows within the window.

If a bound is eventually required, safer alternatives are a maximum accepted history size, a bounded number of new messages per time interval, or a policy that limits which *newly authored* timestamps are allowed without rejecting old messages received by sync. Each changes product semantics and needs an explicit decision.

## 7. Measurements that would settle D6

Benchmark against a temporary on-disk SQLite database using the actual schema and index, with WAL enabled and the same rusqlite build/settings as production. Test at 10k, 100k, and (if realistic) 1M rows:

1. Append at tail (`k ≈ 1`).
2. Insert at 10% from tail (`k ≈ 0.1N`).
3. Insert at beginning (`k ≈ N`).
4. Insert 100 older messages one at a time versus one batch.
5. Repeat while a reader holds a snapshot, and record WAL size/checkpoint behavior.

Record p50/p95 latency, writer queue delay for an unrelated write queued behind rehash, peak WAL size, bytes written if available, and memory/cache behavior. Use a release build and repeat enough times to separate warm-cache from cold-cache results. The core decision threshold should be user-visible pause and writer backlog, not just hash time.

## 8. Compare with the selected per-author-chain direction

The current direction is per-author chains, so a received late message from another author doesn't rehash the global history. A sequence gap makes that author's verified head wait until the missing suffix is fetched. With direct-to-original-author repair, that can wait if the original device is offline; allowing any joined peer with a verified copy to serve that author's signed suffix improves availability without changing the chain format.

The global ledger trades that gap/retry machinery for rehash cost and a global canonical order. Both designs still need stable message identity, signatures, deduplication, transactional batch inserts, and final state verification. A direct performance measurement is needed before claiming one is faster for this particular chat size/workload.

## 9. Bottom line

- Hash computation is negligible; SQLite row/page rewrites dominate.
- Tail appends are cheap; old inserts scale linearly with the remaining suffix.
- 100k full-tail rehashes could mean tens of megabytes of WAL/page traffic per insertion; 1M could mean hundreds of megabytes.
- Sync batches should never rehash once per message.
- A timestamp window is a product-level data-loss trade-off, not a purely technical optimization.
- Benchmark an actual SQLite implementation before using D6 to reject the global ledger. The per-author design avoids that cost but makes availability depend on finding a peer with the author's missing suffix.