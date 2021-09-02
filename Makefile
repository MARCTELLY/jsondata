build:
	# ... build ...
	cargo +nightly build
	cargo +stable build
	# ... test ...
	cargo +nightly test --no-run
	cargo +stable test --no-run
	# ... bench ...
	cargo +nightly bench --no-run
	# ... doc ...
	cargo +nightly doc
	cargo +stable doc
	# ... meta commands ...
	cargo +nightly clippy --all-targets --all-features

flamegraph:
	echo "no executable"

prepare:
	check.sh check.out
	perf.sh perf.out

clean:
	cargo clean
	rm -f check.out perf.out flamegraph.svg perf.data perf.data.old
