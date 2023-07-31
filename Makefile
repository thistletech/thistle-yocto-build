STRIP := "-C link-arg=-s"

check::
	cargo clippy --workspace -- -D warnings
	cargo fmt --all

check-enforce::
	cargo clippy --workspace -- -D warnings
	cargo fmt --all -- --check

run::
	cargo run -- build --dryrun samples/qemuarm64.yml

build::
	cargo build

test::
	cargo test --workspace

release::
	( \
	RUSTFLAGS=${STRIP} cargo build --release --locked --target x86_64-unknown-linux-musl; \
	mkdir -p dist; \
	cp target/x86_64-unknown-linux-musl/release/thistle-yocto-build dist/thistle-yocto-build-x86_64-unknown-linux-musl; \
	cd dist; \
	sha256sum thistle-yocto-build-* > sha256sums.txt; \
	)

ci:: check-enforce build test
	echo "done"

build-scp:: check release
	scp target/x86_64-unknown-linux-musl/release/thistle-yocto-build buildserver:~/

docker-prepare::
	docker build -t thistlebuilder .

docker-build::
	docker run -v ${PWD}/target-ubuntu:/src/target -v ${PWD}:/src  -t thistlebuilder

