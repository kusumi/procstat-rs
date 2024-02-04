bin1:
	cargo build --release --features=curses
	# cargo run --release --features=curses -- ...
bin2:
	cargo build --release --features=stdout
	# cargo run --release --features=stdout -- ...
fmt:
	cargo fmt
	git status
clean:
	cargo clean
test1:
	cargo test --release --features=curses
test2:
	cargo test --release --features=stdout
lint1:
	cargo clippy --release --fix --features=curses
	git status
lint2:
	cargo clippy --release --fix --features=stdout
	git status

xxx1:	fmt lint1 test1
xxx2:	fmt lint2 test2
