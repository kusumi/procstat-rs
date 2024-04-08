bin1:
	cargo build --release --features=curses
	# cargo run --release --features=curses -- ...
bin2:
	cargo build --release --features=stdout
	# cargo run --release --features=stdout -- ...
clean:
	cargo clean
fmt:
	cargo fmt
	git status
lint1:
	cargo clippy --release --fix --all --features=curses
	git status
plint1:
	cargo clippy --release --fix --all --features=curses -- -W clippy::pedantic
	git status
lint2:
	cargo clippy --release --fix --all --features=stdout
	git status
plint2:
	cargo clippy --release --fix --all --features=stdout -- -W clippy::pedantic
	git status
test1:
	cargo test --release --features=curses
test2:
	cargo test --release --features=stdout

xxx1:	fmt lint1 test1
xxx2:	fmt lint2 test2
