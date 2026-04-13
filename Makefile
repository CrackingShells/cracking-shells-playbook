.PHONY: dev-setup build-rust-local

dev-setup:
	git config core.hooksPath .githooks
	chmod +x .githooks/pre-commit
	@if ! command -v uv >/dev/null 2>&1; then \
		echo "Installing uv..."; \
		curl -LsSf https://astral.sh/uv/install.sh | sh; \
	fi
	npm install

# Build dirtree-rdm for the local architecture only.
# CI handles the full cross-compilation matrix.
build-rust-local:
	cd skills/managing-roadmaps/scripts/dirtree-rdm && bash build.sh local
