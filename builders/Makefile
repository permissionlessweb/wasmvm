# 4.0.0-zk uses OUR images only.
# Do not pull or push cosmwasm/libwasmvm-builder:0103-* (rustc 1.86, no Path A).
# See docs/BUILDERS.md and builders/README.md.
ZK_TAG := 4.0.0-zk
GHCR_PREFIX := ghcr.io/terpnetwork

.PHONY: docker-image-alpine-nightly
docker-image-alpine-nightly:
	docker build --platform linux/amd64 . -t terpnetwork/zk-alpine-builder:$(ZK_TAG) -f ./Dockerfile.alpine-nightly

.PHONY: docker-image-debian-nightly
docker-image-debian-nightly:
	docker build --platform linux/amd64 . -t terpnetwork/zk-debian-builder:$(ZK_TAG) -f ./Dockerfile.debian-nightly

.PHONY: docker-image-cross-nightly
docker-image-cross-nightly:
	docker build --platform linux/amd64 . -t terpnetwork/zk-cross-builder:$(ZK_TAG) -f ./Dockerfile.cross-nightly

.PHONY: docker-images-4.0.0-zk
docker-images-4.0.0-zk: docker-image-alpine-nightly docker-image-debian-nightly docker-image-cross-nightly

.PHONY: docker-publish-4.0.0-zk
docker-publish-4.0.0-zk:
	docker tag terpnetwork/zk-alpine-builder:$(ZK_TAG) $(GHCR_PREFIX)/zk-alpine-builder:$(ZK_TAG)
	docker tag terpnetwork/zk-debian-builder:$(ZK_TAG) $(GHCR_PREFIX)/zk-debian-builder:$(ZK_TAG)
	docker tag terpnetwork/zk-cross-builder:$(ZK_TAG) $(GHCR_PREFIX)/zk-cross-builder:$(ZK_TAG)
	docker push $(GHCR_PREFIX)/zk-alpine-builder:$(ZK_TAG)
	docker push $(GHCR_PREFIX)/zk-debian-builder:$(ZK_TAG)
	docker push $(GHCR_PREFIX)/zk-cross-builder:$(ZK_TAG)

# Default `docker-images` used to build CosmWasm 0103. That is the mixed-generation trip.
.PHONY: docker-images
docker-images:
	@echo "ERROR: 4.0.0-zk does not use cosmwasm/libwasmvm-builder:0103-*." >&2
	@echo "Build ours:  make docker-images-4.0.0-zk" >&2
	@echo "Push ours:   make docker-publish-4.0.0-zk  (ghcr.io/terpnetwork/zk-*-builder:4.0.0-zk)" >&2
	@echo "Upstream 0103 (do not use for Path A): make docker-images-upstream-0103" >&2
	@exit 1

.PHONY: docker-publish
docker-publish: docker-images

# Historical CosmWasm 0103 (rustc 1.86). Not Path A. Not 4.0.0-zk.
BUILDERS_PREFIX := cosmwasm/libwasmvm-builder:0103

.PHONY: docker-image-debian
docker-image-debian:
	docker build --pull . -t $(BUILDERS_PREFIX)-debian -f ./Dockerfile.debian

.PHONY: docker-image-cross
docker-image-cross:
	docker build --pull . -t $(BUILDERS_PREFIX)-cross -f ./Dockerfile.cross

.PHONY: docker-image-alpine
docker-image-alpine:
	docker build --pull . -t $(BUILDERS_PREFIX)-alpine -f ./Dockerfile.alpine

.PHONY: docker-images-upstream-0103
docker-images-upstream-0103: docker-image-debian docker-image-cross docker-image-alpine
