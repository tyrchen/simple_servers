TOP_DIR=.
MEMBERS=common echo noise

build: format
	@echo "Building the software..."
	@cargo build --verbose --all

format:
	@cargo fmt -- --check

init: install dep
	@echo "Initializing the repo..."

travis-init:
	@echo "Initialize software required for travis (normally ubuntu software)"

install:
	@echo "Install software required for this repo..."

dep:
	@echo "Install dependencies required for this repo..."

pre-build: dep
	@echo "Running scripts before the build..."
	@cargo clippy -- -D warnings

post-build: doc
	@echo "Running scripts after the build is done..."

all: pre-build build post-build

test:
	@echo "Running test suites..."
	@cargo test --verbose --all

doc:
	@echo "Building the documentation..."
	@cargo doc --no-deps --all-features

precommit: pre-build build post-build

travis: precommit

travis-deploy:
	@echo "Deploy the software by travis"
	@make release

clean:
	@echo "Cleaning the build..."

run:
	@echo "Running the software..."
	@iex -S mix

submodule:
	@git submodule update --init --recursive

cloc:
	@cloc $(MEMBERS)

include .makefiles/*.mk

.PHONY: build init travis-init install dep pre-build post-build all test dialyzer doc precommit travis clean watch run bump-version create-pr submodule cloc
