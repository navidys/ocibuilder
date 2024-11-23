# This Makefile is intended for developer convenience.  For the most part
# all the targets here simply wrap calls to the `cargo` tool.  Therefore,
# most targets must be marked 'PHONY' to prevent `make` getting in the way
#
#prog :=xnixperms

DESTDIR ?=

SELINUXOPT ?= $(shell test -x /usr/sbin/selinuxenabled && selinuxenabled && echo -Z)
# Get crate version by parsing the line that starts with version.
CRATE_VERSION ?= $(shell grep ^version Cargo.toml | awk '{print $$3}')
GIT_TAG ?= $(shell git describe --tags)

# Set this to any non-empty string to enable unoptimized
# build w/ debugging features.
debug ?=

# Set path to cargo executable
CARGO ?= cargo

PKG_MANAGER ?= $(shell command -v dnf yum|head -n1)
PRE_COMMIT = $(shell command -v bin/venv/bin/pre-commit ~/.local/bin/pre-commit pre-commit | head -n1)

# All complication artifacts, including dependencies and intermediates
# will be stored here, for all architectures.  Use a non-default name
# since the (default) 'target' is used/referenced ambiguously in many
# places in the tool-chain (including 'make' itself).
CARGO_TARGET_DIR ?= targets
export CARGO_TARGET_DIR  # 'cargo' is sensitive to this env. var. value.

ifdef debug
$(info debug is $(debug))
  # These affect both $(CARGO_TARGET_DIR) layout and contents
  # Ref: https://doc.rust-lang.org/cargo/guide/build-cache.html
  release :=
  profile :=debug
else
  release :=--release
  profile :=release
endif

.PHONY: all
all: binary

bin:
	mkdir -p $@

$(CARGO_TARGET_DIR):
	mkdir -p $@

.PHONY: binary
binary: bin $(CARGO_TARGET_DIR) ## Build ocibuilder binary
	$(CARGO) build $(release)
	cp $(CARGO_TARGET_DIR)/$(profile)/ocibuilder bin/ocibuilder$(if $(debug),.debug,)

.PHONY: clean
clean: ## Cleanup
	rm -rf bin target
	if [ "$(CARGO_TARGET_DIR)" = "targets" ]; then rm -rf targets; fi

.PHONY: install
install: ## Install ocibuilder binary
	install ${SELINUXOPT} -D -m0755 bin/ocibuilder $(DESTDIR)/ocibuilder

.PHONY: uninstall
uninstall: ## Uninstall ocibuilder binary
	rm -f $(DESTDIR)/ocibuilder

#=================================================
# Testing and validation
#=================================================

.PHONY: test
test: $(CARGO_TARGET_DIR) ## Run builder tests
	$(CARGO) test --test builder

.PHONY: validate
validate: validate.cargo pre-commit codespell ## Validate all including cargo

.PHONY: validate.cargo
validate.cargo: $(CARGO_TARGET_DIR) ## Cargo fmt and clippy validation
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy -p ocibuilder@$(CRATE_VERSION) -- -D warnings

.PHONY: pre-commit
pre-commit:   ## Run pre-commit
ifeq ($(PRE_COMMIT),)
	@echo "FATAL: pre-commit was not found, make .install.pre-commit to installing it." >&2
	@exit 2
endif
	$(PRE_COMMIT) run -a

.PHONY: codespell
codespell: ## Run codespell
	@echo "running codespell"
	@codespell -S ./target,./targets -L crate

#=================================================
# Required tools installation tartgets
#=================================================

.PHONY: install.tools
install.tools: .install.pre-commit .install.codespell ## Install needed tools

.PHONY: .install.pre-commit
.install.pre-commit:
	if [ -z "$(PRE_COMMIT)" ]; then \
		python3 -m pip install --user pre-commit; \
	fi

.PHONY: .install.codespell
.install.codespell:
	sudo ${PKG_MANAGER} -y install codespell

#=================================================
# Help menu
#=================================================

_HLP_TGTS_RX = '^[[:print:]]+:.*?\#\# .*$$'
_HLP_TGTS_CMD = grep -E $(_HLP_TGTS_RX) $(MAKEFILE_LIST)
_HLP_TGTS_LEN = $(shell $(_HLP_TGTS_CMD) | cut -d : -f 1 | wc -L)
_HLPFMT = "%-$(_HLP_TGTS_LEN)s %s\n"
.PHONY: help
help: ## Print listing of key targets with their descriptions
	@printf $(_HLPFMT) "Target:" "Description:"
	@printf $(_HLPFMT) "--------------" "--------------------"
	@$(_HLP_TGTS_CMD) | sort | \
		awk 'BEGIN {FS = ":(.*)?## "}; \
			{printf $(_HLPFMT), $$1, $$2}'
