
## Zk-Diagram
```
┌─────────────────────────────────────────────────────────────────────────┐
│                    HALO2 VERIFYING KEY LIFECYCLE                         │
│                   CosmWasm + Zero-Knowledge Integration                  │
└─────────────────────────────────────────────────────────────────────────┘

┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ PHASE 1: BUILD & GENERATE (Off-chain, Developer Machine)              ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

    ┌──────────────────┐
    │  Rust Circuit    │
    │  Definition      │
    │  (Halo2)         │
    └────────┬─────────┘
             │
             │ cargo build --release
             ▼
    ┌──────────────────┐         ┌──────────────────┐
    │  contract.wasm   │         │  build_and_write()│
    │  (CosmWasm)      │         │  (Proving Key)   │
    └────────┬─────────┘         └────────┬─────────┘
             │                            │
             │                            │ Generates
             │                            ▼
             │                   ┌──────────────────┐
             │                   │  vk_binary.bin   │
             │                   │                  │
             │                   │ [Params Bytes]   │
             │                   │ [VK Bytes]       │
             │                   └────────┬─────────┘
             │                            │
             └────────────┬───────────────┘
                          │
                          │ Developer prepares upload
                          ▼

┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ PHASE 2: UPLOAD (On-chain Transaction via CLI/API)                    ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

    $ wasmd tx wasm store-with-vk contract.wasm vk_binary.bin \
        --from deployer --gas auto
             │
             │ CLI Command
             ▼
    ┌──────────────────────────────────────────┐
    │  MsgStoreCodeWithVk                      │
    │  {                                       │
    │    wasm_byte_code: [...]                 │
    │    vk_byte_code: [Params][VK]           │
    │    instantiate_permission: {...}         │
    │  }                                       │
    └──────────────────┬───────────────────────┘
                       │
                       │ Blockchain Transaction
                       ▼
    ┌──────────────────────────────────────────┐
    │  Keeper: StoreCodeWithVk()               │
    │                                          │
    │  1. Validate WASM                        │
    │  2. validate_vk_bytes(vk_byte_code)     │
    │  3. Compute checksums                    │
    │     - wasm_checksum = sha256(wasm)      │
    │     - vk_checksum = sha256(vk)          │
    │  4. Store on-chain metadata             │
    │  5. Emit CodeIDWithVK                   │
    └──────────────────┬───────────────────────┘
                       │
                       │ State Update
                       ▼
    ┌──────────────────────────────────────────┐
    │  Blockchain State                        │
    │                                          │
    │  CodeInfo {                              │
    │    code_id: 42                          │
    │    creator: "cosmos1..."                │
    │    wasm_checksum: 0xABCD...             │
    │    vk_checksum: 0x1234...               │
    │  }                                       │
    └──────────────────┬───────────────────────┘
                       │
                       │ Event: CodeStoredWithVK
                       ▼

┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ PHASE 3: CACHE (Node Startup / First Use)                             ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

    Node starts or first instantiate with code_id=42
             │
             │ Cache miss
             ▼
    ┌──────────────────────────────────────────┐
    │  FileSystemCache::save_wasm_with_vk()    │
    │                                          │
    │  Downloads from blockchain state         │
    └──────────────────┬───────────────────────┘
                       │
                       │ Write to disk
                       ▼
    ┌──────────────────────────────────────────┐
    │  Filesystem Cache                        │
    │  ~/.wasmd/cache/wasm/                    │
    │                                          │
    │  ├─ modules/                             │
    │  │  └─ v1-<checksum>.module              │
    │  │     (compiled WASM)                   │
    │  │                                       │
    │  └─ vks/                                 │
    │     └─ <checksum>.vk                     │
    │        [circuit_type: u8]                │
    │        [Params bytes]                    │
    │        [VK bytes]                        │
    └──────────────────┬───────────────────────┘
                       │
                       │ Persisted to disk
                       ▼

┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ PHASE 4: PIN (Memory Optimization for Hot VKs)                        ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

    Contract execution needs VK
             │
             │ High-frequency verification
             ▼
    ┌──────────────────────────────────────────┐
    │  Cache::pin_vk(checksum)                 │
    │                                          │
    │  1. load_vk_from_disk()                  │
    │     ├─ Read file from cache              │
    │     ├─ Parse circuit_type (1 byte)       │
    │     └─ Extract VK bytes                  │
    │                                          │
    │  2. LoadedVerifyingKey::from_bytes()     │
    │     ├─ Deserialize Params                │
    │     │   let params = Params::read()      │
    │     └─ Deserialize VK (circuit-specific) │
    │         match circuit_type {             │
    │           Orchard => VK::read<Orchard>() │
    │           Other => VK::read<Other>()     │
    │         }                                │
    │                                          │
    │  3. Arc::new(loaded_vk)                  │
    │     Store in pinned_vk_cache             │
    └──────────────────┬───────────────────────┘
                       │
                       │ Pinned in RAM
                       ▼
    ┌──────────────────────────────────────────┐
    │  Memory Cache (LRU + Pinned)             │
    │                                          │
    │  pinned_vk_cache: HashMap {              │
    │    checksum_1 → Arc<LoadedVK> ──┐       │
    │    checksum_2 → Arc<LoadedVK>   │       │
    │  }                              │       │
    │                                 │       │
    │  ┌──────────────────────────────▼─────┐ │
    │  │  LoadedVerifyingKey              │ │
    │  │  {                                 │ │
    │  │    params: Params<vesta::Affine>  │ │
    │  │    vk: VerifyingKey<vesta::Affine>│ │
    │  │    original_size: usize            │ │
    │  │  }                                 │ │
    │  └────────────────────────────────────┘ │
    │                                          │
    │  Total pinned memory: 24.3 MB            │
    └──────────────────┬───────────────────────┘
                       │
                       │ Ready for instant access
                       ▼

┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ PHASE 5: VERIFY (Contract Execution with ZK Proof)                    ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

    User calls: verify_proof(proof_bytes)
             │
             │ Contract execution
             ▼
    ┌──────────────────────────────────────────┐
    │  WASM Contract (Rust)                    │
    │                                          │
    │  #[entry_point]                          │
    │  pub fn execute(                         │
    │    deps: DepsMut,                        │
    │    env: Env,                             │
    │    msg: ExecuteMsg                       │
    │  ) -> Result<Response> {                 │
    │    match msg {                           │
    │      Verify { proof } => {               │
    │        deps.api.verify_halo2_proof(      │
    │          &proof,                         │
    │          &public_inputs                  │
    │        )?;                               │
    │      }                                   │
    │    }                                     │
    │  }                                       │
    └──────────────────┬───────────────────────┘
                       │
                       │ Host function call
                       ▼
    ┌──────────────────────────────────────────┐
    │  wasmvm: do_verify_halo2_proof()         │
    │                                          │
    │  1. Get cached LoadedVK (from pinned)    │
    │     let vk = cache.get_pinned_vk()      │
    │                                          │
    │  2. Deserialize proof bytes              │
    │     let proof = Proof::read(proof_bytes) │
    │                                          │
    │  3. Parse public inputs                  │
    │     let public_inputs = [...];          │
    │                                          │
    │  4. Create verifier strategy             │
    │     let strategy = SingleStrategy::new(  │
    │       &vk.params                         │
    │     );                                   │
    │                                          │
    │  5. VERIFY THE PROOF                     │
    │     verify_proof(                        │
    │       &vk.params,                        │
    │       &vk.vk,                            │
    │       strategy,                          │
    │       &[&[&public_inputs]],             │
    │       &mut transcript                    │
    │     )?;                                  │
    │                                          │
    │  ✓ Proof is valid                        │
    └──────────────────┬───────────────────────┘
                       │
                       │ Return success
                       ▼
    ┌──────────────────────────────────────────┐
    │  Contract Response                       │
    │                                          │
    │  Response::new()                         │
    │    .add_attribute("action", "verify")    │
    │    .add_attribute("status", "success")   │
    └──────────────────┬───────────────────────┘
                       │
                       │ Transaction success
                       ▼
    ┌──────────────────────────────────────────┐
    │  Blockchain State Updated                │
    │                                          │
    │  ✓ Zero-knowledge proof verified         │
    │  ✓ Privacy preserved                     │
    │  ✓ Computation validated                 │
    └──────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│  KEY INNOVATIONS                                                         │
│  • Upload once, verify everywhere (blockchain distribution)              │
│  • Memory-efficient caching with LRU + pinning                          │
│  • Circuit-type aware deserialization                                    │
│  • Zero-knowledge proofs in CosmWasm smart contracts                    │
│  • Gas-metered cryptographic verification                                │
└─────────────────────────────────────────────────────────────────────────┘
