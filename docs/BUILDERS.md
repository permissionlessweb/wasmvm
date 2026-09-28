# Libwasmvm builders (4.0.0-zk)

Dynamic libraries (`libwasmvm.{so,dylib}`) and muslc archives
(`libwasmvm_muslc.*.a`) for **4.0.0-zk** are built with **our** images.

Do **not** `docker pull cosmwasm/libwasmvm-builder:0103-*`. Those are rustc
1.86, have no Path A (Stwo `portable_simd` needs nightly), and mixing them
with a 4.0.0-zk recut is how Linux `go test` failed (`store_param` undefined)
against a leftover 3.0.7 `.so`.

| Image | Dockerfile | Output |
|-------|------------|--------|
| `terpnetwork/zk-alpine-builder:4.0.0-zk` | `builders/Dockerfile.alpine-nightly` | `libwasmvm_muslc.{x86_64,aarch64}.a` |
| `terpnetwork/zk-debian-builder:4.0.0-zk` | `builders/Dockerfile.debian-nightly` | `libwasmvm.{x86_64,aarch64}.so` |
| `terpnetwork/zk-cross-builder:4.0.0-zk` | `builders/Dockerfile.cross-nightly` | osxcross dylib (optional) |

Local tags only: `terpnetwork/zk-*-builder:4.0.0-zk`. Do **not** push these
images (2–8GB toolchains). Operator registry is
`registry.terp.network/terp-core:<tag>` for compiled `terpd`, and S3
`releases/zk-wasmvm/v4.0.0-zk/` for muslc. Not GHCR.

## Build the images (once)

```sh
cd crates/zk-wasmvm/builders
make docker-images-4.0.0-zk
# make docker-images           # ERROR: refuses CosmWasm 0103
# make docker-publish-4.0.0-zk # ERROR: builders are not published
```

`make docker-images` is fail-closed. Upstream 0103 Dockerfiles remain as
`make docker-images-upstream-0103` and are not Path A.

## Recut host libs

From `crates/zk-wasmvm`:

```sh
make release-build    # alpine + linux + macos, then verify-libwasmvm
make verify-libwasmvm
```

From terp-core:

```sh
make wasmvm-release-build
make wasmvm-verify
```

Linux `go test` links `internal/api/libwasmvm.$(arch).so`, **not** muslc.
Recutting only alpine is the mixed-generation trip. `verify-libwasmvm`
requires muslc + `.so` + dylib all export `store_param`, and muslc must
contain `stwo: Dummy DSTW rejected`.

**macOS dylib:** native `make build-libwasmvm` on Darwin. osxcross
`x86_64-apple-darwin` still fails Path A (`__rust_probestack`). Do not
replace a good native `libwasmvm.dylib` with a failed lipo.

## Debian aarch64 sysroot

`guest/build_gnu_aarch64.sh` uses `clang --sysroot=/usr/aarch64-linux-gnu`.
`Dockerfile.debian-nightly` **must** install `libc6-dev-arm64-cross` (and
`linux-libc-dev-arm64-cross`). `--no-install-recommends` does not pull
those; without them blake3 neon fails with `'assert.h' file not found`.
The image build `test -f /usr/aarch64-linux-gnu/include/assert.h`.

## S3 (muslc; Cosmovisor ELFs)

Muslc `.a` is gitignored (`/internal/api/lib*.a`). Operators and fresh-VM
fetch it from:

`https://s3.terp.network/releases/zk-wasmvm/v4.0.0-zk/`

(`fetch_zk_muslc.sh`, default `WASMVM_MUSLC_BASE`). glibc `.so` and Darwin
dylib are committed in this repo so `go test` links one generation.
