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
lint1:
	cargo clippy --release --fix --features=curses
	git status
lint2:
	cargo clippy --release --fix --features=stdout
	git status
