# Bubblegum → MPL Core Migration Design

## Goal

CLI tool to migrate Bubblegum compressed NFT collections to MPL Core assets with agent identity registration. Run by the collection authority.

## CLI Interface

```
mpl-agent-migrator migrate \
  --collection <source-bubblegum-collection> \
  --source bubblegum \
  --destination <existing-core-collection>   # optional, creates new if omitted
  --agent-uri <url>                          # optional, overrides default agent data
  --burn                                     # optional, burns originals after migration
  --batch-size 10                            # assets per processing batch
  --keypair ~/.config/solana/id.json \
  --rpc <url> \
  --das <url> \
  --execute                                  # required to send transactions
```

Without `--execute`: dry run. Fetches assets, validates authority, prints plan, exits.
With `--execute`: sends transactions.

## Migration Flow

### 1. Fetch all source assets
- Page through DAS `getAssetsByGroup` to collect every compressed NFT in the collection
- Collect: asset ID, owner, name, URI, compression proof data

### 2. Validate authority
- Verify the signer (keypair) is the collection authority / tree creator
- Fail fast if not authorized

### 3. Create destination collection (if no --destination)
- Fetch source collection metadata via DAS `getAsset`
- Call mpl-core `createCollection` with same name/URI
- Signer becomes update authority

### 4. Per-asset migration (1 transaction each)
For each source asset, build and send a single transaction containing:
- `createV2` — new Core asset with original name, URI, owner set to original holder
- `registerIdentityV1` — register agent identity with `--agent-uri` or default registration data

### 5. Optional burn (separate transaction per asset)
If `--burn` flag:
- Use `getAssetWithProof` from Bubblegum SDK for merkle proof
- Call Bubblegum `burn` for each original compressed NFT
- Requires tree authority

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Original NFT handling | Leave alone (burn optional via --burn) | Safest default, avoids requiring tree authority |
| Authority model | Collection authority required | Migration is a project-level operation |
| Metadata | Mirror original name + URI | Original off-chain JSON carries over |
| Agent identity | Always registered | Core purpose of migration tool |
| Agent URI | Default data, overridable via --agent-uri | Sensible default TBD |
| Execution mode | Dry run by default, --execute to send | Safer default |
| Create + register | Single transaction per asset | Atomic — no orphaned assets without identity |

## Error Handling

- Transaction failure: log which asset failed, continue to next
- End summary: succeeded / failed / skipped counts
- Idempotent: before creating, check if Core asset already exists for owner+name in destination collection. Skip if found.
- Safe to re-run after partial failures

## Pagination

DAS returns paginated results. Fetch all pages upfront using `page` cursor before executing any transactions. This builds the complete migration plan for dry run display and accurate progress tracking.
