bin1:
	cargo build --features=curses
	# cargo run --features=curses -- ...
bin2:
	cargo build --features=stdout
	# cargo run --features=stdout -- ...
fmt:
	cargo fmt
	git status
clean:
	cargo clean
lint1:
	cargo clippy --fix --features=curses
	git status
lint2:
	cargo clippy --fix --features=stdout
	git status
