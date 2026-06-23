# MPL Agent Reputation

Reputation registry for MPL Agent. Mints soulbound, on-chain reviews tied to a
work-receipt cNFT that was previously minted by [`mpl-agent-tools`](../mpl-agent-tools).

**Program ID:** `REPREG5c1gPHuHukEyANpksLdHFaJCiTrm6zJgNhRZR`

---

## Overview

This program produces two kinds of on-chain state:

1. **Review cNFTs** — Bubblegum V2 compressed NFTs minted into a soulbound
   `reviews_collection` MPL Core collection. Each review carries the star
   rating in its name (`"Agent Feedback (5★)"`) and a feedback URI pointing
   to off-chain JSON.
2. **`ReviewRecordV1` PDAs** — One per `(work-receipt, review)` pair. The
   PDA's existence is the on-chain proof that "this receipt has been
   reviewed" and gates against double-review.

Reviews are gated by **work receipts**. A reviewer must hold a Bubblegum
cNFT receipt (minted by `mpl-agent-tools::MintWorkReceiptV1` into the
canonical `receipts_collection`) and supply a Merkle proof. The program
reconstructs the receipt's leaf hash on-chain using the agent asset key as
the receipt's creator — preventing replay of one agent's receipt against a
review for another agent.

---

## State

### `ReviewRecordV1` PDA

Seeds: `["review_record", receipt_asset_id]` (where `receipt_asset_id` is
the Bubblegum asset id of the work-receipt cNFT).

| Field              | Type      | Description                                  |
|--------------------|-----------|----------------------------------------------|
| `key`              | `u8`      | Account discriminator                        |
| `bump`             | `u8`      | PDA bump seed                                |
| `_padding`         | `[u8; 6]` | Alignment                                    |
| `reviewer`         | `Pubkey`  | Wallet that left the review                  |
| `receipt_asset_id` | `Pubkey`  | Bubblegum asset id of the reviewed receipt   |

**Size:** 72 bytes.

### Canonical stateless PDAs

| Name                | Seeds                              | Owner                  |
|---------------------|------------------------------------|------------------------|
| Reviews collection  | `["reviews_collection"]`           | MPL Core               |
| Reviews authority   | `["reviews_authority"]`            | Program signer (no data) |
| Reviews tree #N     | `["reviews_tree", index_le]`       | MPL Account Compression |

The **reviews authority** PDA is the `update_authority` on the reviews
collection AND the `tree_creator` on every reviews merkle tree. It signs
every MintV2 CPI via `invoke_signed` — neither the original collection
nor the original tree creator is ever needed again.

The receipts side has the symmetric set of PDAs in
`mpl-agent-tools` (`receipts_collection`, `receipts_authority`,
`receipts_tree`).

---

## Discovery Patterns

The discovery surface is split across three sources:

- **`getProgramAccounts`** on `mpl-agent-reputation` (and `mpl-agent-tools`)
  for the PDAs the programs own (`ReviewRecordV1`, `ExecutiveProfileV1`,
  `ExecutionDelegateRecordV1`).
- **DAS (`searchAssets`, `getAssetsByOwner`, `getAssetsByGroup`)** for the
  cNFTs (receipts and reviews) and their off-chain metadata JSON.
- **Local PDA derivation** when the seeds are known — most addresses in
  the system are derivable without an RPC call.

Each user story below is tagged with an importance ranking based on how
often a typical reputation-aware product would call it, plus the RPC
cost it takes today (no proposed additions). Roughly:
★★★★★ = on every relevant page; ★★★★☆ = on most product paths;
★★★☆☆ = niche dashboards or composition; ★★☆☆☆ = indexer / fraud
only.

In the "RPC calls" column: **DAS** = one call to a DAS-capable RPC
(`searchAssets` / `getAssetsByOwner`); **gPA** = `getProgramAccounts`;
`getMultipleAccounts` is the batched form of `getAccount` and is
treated as 1 call regardless of N. `N` is the per-agent or per-wallet
fan-out (number of receipts, agents, etc.).

| # | Story                                      | RPC calls (today)                          | Importance |
|---|--------------------------------------------|--------------------------------------------|------------|
| 1b| Reputation summary / all reviews for agent | 1 DAS (aggregated client-side)             | ★★★★★      |
| 1a| Work the agent has done                    | 1 DAS                                      | ★★★★☆      |
| 3 | Reviews a client can leave                 | 1 DAS + 1 `getMultipleAccounts`            | ★★★★☆      |
| 6 | Has a specific receipt been reviewed?      | 1 `getAccount`                             | ★★★★☆      |
| 2 | Outstanding (unreviewed) work for an agent | 1 DAS + 1 `getMultipleAccounts`            | ★★★☆☆      |
| 4 | Reviews a particular wallet has left       | 1 gPA                                      | ★★★☆☆      |
| 5 | All activity for a human user              | 3 gPA + (1 + N) DAS                        | ★★★☆☆      |
| 7 | Verify a review is canonical               | 1 `getAccount`                             | ★★☆☆☆      |

Below are the end-to-end user stories.

### 1. "Show me everything an agent has done and how it's rated"

#### 1a. Work the agent has done (all receipt cNFTs)

**Importance:** ★★★★☆ — agent profile pages, marketplace listings,
"experience" panels. Common but lower per-page than 1c.

Receipts always carry `creators = [{address: agent_asset, share: 100}]` and
`collection = receipts_collection`, so DAS can return them in one call.

```ts
const receipts = await das.searchAssets({
  grouping: ['collection', receiptsCollection.toString()],
  creatorAddress: agentAsset.toString(),
  creatorVerified: false,
  limit: 1000,
});
```

Each result is a Bubblegum V2 cNFT owned by the client wallet that received
the work, with `content.json_uri` pointing to the receipt JSON.

#### 1b. All reviews left for an agent (also the reputation summary)

**Importance:** ★★★★★ — the headline reputation view. Every agent
profile, every detail page, every "explain this score" drill-down hits
this. Rolled-up stats (avg rating, count, histogram) are aggregated
client-side from the same DAS response, so there's no separate "summary"
fetch.

`LeaveReviewV1` enforces `leaf_owner == agent_asset.owner` on-chain, so
every review cNFT for an agent is owned by the wallet that controls the
agent at review time. The full set of reviews for that wallet is one
DAS call:

```ts
const reviews = await das.getAssetsByOwner({
  ownerAddress: agentWallet.toString(),
});

const forReviewsCollection = reviews.items.filter((a) =>
  a.grouping?.some(
    (g) =>
      g.group_key === 'collection' &&
      g.group_value === reviewsCollection.toString(),
  ),
);
```

**Edge case — wallet owns multiple agent assets:** the result above
contains reviews for *all* of the wallet's agents commingled. To
attribute each review to a specific `agent_asset`, look up the
`ReviewRecordV1` PDA referenced by each review (its `receipt_asset_id`
points at the receipt, whose `creators[0].address` is the agent). A
future change could swap `leaf_owner` to MPL Core's `AssetSigner` PDA
(`["mpl-core-execute", agent_asset]`) so the asset itself owns its
reviews and no two agents ever commingle.

### 2. "What outstanding (unreviewed) work does an agent have?"

**Importance:** ★★★☆☆ — agent-operator dashboards ("nudge your clients
to leave a review"). Not user-facing on the discovery side.

For an agent, the set of unreviewed receipts is:

```
{ receipts where creator == agent_asset } \ { receipts where ReviewRecordV1["review_record", receipt_asset_id] exists }
```

Concretely:

```ts
const receipts = await das.searchAssets({
  grouping: ['collection', receiptsCollection.toString()],
  creatorAddress: agentAsset.toString(),
  creatorVerified: false,
});

const outstanding = [];
for (const r of receipts.items) {
  const recordPda = findReviewRecordV1Pda(umi, {
    receiptAssetId: r.id,
  });
  const account = await umi.rpc.getAccount(recordPda);
  if (!account.exists) outstanding.push(r);
}
```

This is the "reputation pipeline" view for an agent — work delivered but
not yet rated.

### 3. "What reviews can a human user (client) leave?"

**Importance:** ★★★★☆ — drives the client-side "leave a review"
inbox / notification badge. Hit on every client dashboard load.

Symmetric to (2), but from the *client* wallet's perspective: every
receipt the client owns is a review the client could write.

```ts
const owned = await das.getAssetsByOwner({
  ownerAddress: clientWallet.toString(),
});

const reviewable = owned.items
  .filter((a) => a.grouping?.some((g) =>
    g.group_key === 'collection' && g.group_value === receiptsCollection.toString()
  ));

const stillReviewable = [];
for (const r of reviewable) {
  const recordPda = findReviewRecordV1Pda(umi, { receiptAssetId: r.id });
  if (!(await umi.rpc.getAccount(recordPda)).exists) {
    stillReviewable.push(r);
  }
}
```

### 4. "What reviews has a particular wallet left?"

**Importance:** ★★★☆☆ — reviewer profile pages, reviewer-trust signals
("is this reviewer a serial 1-star bomber?"). Indexer-heavy, not on
every page.

A wallet that has left reviews is the `reviewer` field on its
`ReviewRecordV1` PDAs. One `getProgramAccounts` call with a memcmp filter
on the `reviewer` offset returns them.

```ts
const records = await umi.rpc.getProgramAccounts(mplAgentReputationProgramId, {
  filters: [
    // Account discriminator (Key::ReviewRecordV1 = 1)
    { memcmp: { offset: 0, bytes: bs58encode(new Uint8Array([1])) }},
    // reviewer pubkey at offset 8 (1 key + 1 bump + 6 padding)
    { memcmp: { offset: 8, bytes: reviewerWallet.toString() }},
  ],
});
```

Each record gives you `receipt_asset_id`; cross-reference DAS to fetch the
receipt being reviewed, and look up the review cNFT itself in the reviews
collection owned by the agent's wallet.

### 5. "Show all activity for a specific human user"

**Importance:** ★★★☆☆ — wallet/profile pages aggregating receipts,
reviews, executives, delegations. A composition of 3, 4, and
tools-program lookups — rarely a single product call.

Wallets aren't first-class in this system (no human profile PDA), but
their activity is fully discoverable via:

- **Receipts they own** (work done *for* them): `getAssetsByOwner` →
  filter `collection = receipts_collection` (see 3).
- **Reviews they've left**: `getProgramAccounts` on
  `ReviewRecordV1.reviewer` (see 4).
- **Reviews left *about* a wallet's agent**: derive the agent assets the
  wallet owns (via `getAssetsByOwner` filtered to an agent collection),
  then run discovery 1b for each.
- **Executive profiles they control**: `getProgramAccounts` on
  `mpl-agent-tools` with `ExecutiveProfileV1.authority` memcmp filter.
- **Delegations they've granted**: `getProgramAccounts` on
  `mpl-agent-tools` with `ExecutionDelegateRecordV1.agent_asset` /
  `.authority` memcmp filters.

### 6. "Has a specific receipt been reviewed?"

**Importance:** ★★★★☆ — cheap and ubiquitous. Gates every "Leave a
review" CTA in (3); shows the ✓ / ✗ marker next to each receipt in (2);
runs in tight loops inside indexers. Trivial cost so it's called
freely.

```ts
const recordPda = findReviewRecordV1Pda(umi, { receiptAssetId });
const account = await umi.rpc.getAccount(recordPda);
const reviewed = account.exists;
```

If you only need *existence*, no DAS call is required — pure PDA
derivation + one RPC call.

### 7. "Verify a review is canonical"

**Importance:** ★★☆☆☆ — important when it matters (fraud detection,
indexer ingestion, anti-spoof for review aggregators) but not in any UI
hot path. Run once per review at ingest time, not per page view.

Given a candidate review cNFT:

1. Confirm its `collection` group is the canonical reviews collection PDA
   (`["reviews_collection"]` under the reputation program id).
2. Confirm it lives on a tree whose pubkey matches
   `["reviews_tree", index_le]` for some index.
3. Confirm the `ReviewRecordV1` PDA at
   `["review_record", receipt_asset_id]` exists and its `reviewer` ==
   the review's `creators[0].address`.

The same template applies to receipts under the tools program.

---

## Proposed Additions

The fields below would materially improve indexability without altering
the trust model.

### `ReviewRecordV1` — add 2–4 fields

| Field                    | Type     | Why                                                         |
|--------------------------|----------|-------------------------------------------------------------|
| `created_at`             | `i64`    | Ordering, recency decay, dispute windows                    |
| `rating`                 | `u8`     | Aggregates / indexer reads without re-fetching the cNFT     |
| `review_asset_id`        | `Pubkey` | Pointer from record → review cNFT (DAS lookup)              |
| `receipts_merkle_tree`   | `Pubkey` | Pointer from record → receipt cNFT (DAS lookup)             |

`agent_asset` is intentionally *not* on this list: the agent's wallet is
already the `leaf_owner` of every review it has received, so DAS
(`getAssetsByOwner` + collection filter, see 1b) covers the primary
discovery path without an on-chain field.

### `AgentReputationV1` PDA — optional aggregate

A future per-agent PDA at `["agent_reputation", asset_pubkey]` could
roll up review stats so the headline "n reviews, x.x avg" badge becomes a
single `getAccountInfo` instead of a DAS scan:

| Field                | Type        | Why                                  |
|----------------------|-------------|--------------------------------------|
| `review_count`       | `u32`       | "/n reviews" UI                      |
| `rating_sum`         | `u64`       | Average = sum / count                |
| `rating_buckets`     | `[u32; 5]`  | Histogram UI                         |
| `first_reviewed_at`  | `i64`       | Account-age signal                   |
| `last_reviewed_at`   | `i64`       | Activity recency                     |

`LeaveReviewV1` would init-if-needed and bump. Not currently shipped —
discovery 1b aggregates the same data client-side.

### Explicitly out of scope (PoC)

- Agent replies / counter-reviews
- Dispute and revocation (reviews are soulbound by design)
- Reviewer-trust scoring
- On-chain `receipts_minted` counter — would require cross-program CPI
  from `mpl-agent-tools` into `mpl-agent-reputation` on every mint; cheaper
  to compute off-chain.

---

## Instructions

### `CreateReviewsCollectionV1` — permissionless idempotent bootstrap

Creates the canonical reviews collection at `["reviews_collection"]` with
`update_authority = ["reviews_authority"]`. Anyone may call. A hostile
first caller cannot capture authority because it's program-derived. A
second call fails because the collection account already exists.

### `RegisterReviewsTreeV1` — permissionless tree registration

Caller picks an unused `tree_index`, pays the rent. Tree is created at
`["reviews_tree", index_le]`; Bubblegum is configured with `tree_creator
= ["reviews_authority"]`. First-come-first-served — racing callers for
the same index lose the `CreateAccount` step.

### `LeaveReviewV1` — mint a review

The reviewer:
1. Owns a work-receipt cNFT minted by `mpl-agent-tools::MintWorkReceiptV1`.
2. Supplies the receipt's Bubblegum leaf nonce, index, data_hash,
   asset_data_hash, flags + the current Merkle root + a proof path as
   remaining accounts.

The program:
1. Validates all four canonical PDAs (reviews collection / authority /
   tree, plus the receipts collection from the tools program).
2. Reconstructs the receipt's `LeafSchemaV2` hash, computing
   `creator_hash` from `ctx.accounts.asset.key` (the reviewed agent) so
   receipts cannot be replayed against a different agent.
3. CPIs into MPL Account Compression's `verify_leaf` to prove the
   receipt is in its tree.
4. Creates `ReviewRecordV1` PDA — the idempotency gate.
5. Mints a soulbound review cNFT to the agent's wallet via Bubblegum
   `MintV2`, signed by the reviews authority PDA.

---

## Building & Testing

```sh
# Build all programs
pnpm programs:build

# Run program-side tests
pnpm programs:test

# Regenerate IDLs + clients (after any program change)
pnpm generate

# JS client tests (uses local Amman validator)
pnpm clients:js:test

# Rust client tests
pnpm clients:rust:test
```
