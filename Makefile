.PHONY: test clippy clippy-fix format format-check

test:
	$(info ****************** running tests ******************)
	cargo test

clippy:
	$(info ****************** running clippy in check mode ******************)
	cargo clippy

clippy-fix:
	$(info ****************** running clippy in fix mode ******************)
	cargo clippy --fix --bin "personal-wiki"

format:
	$(info ****************** running rustfmt in fix mode ******************)
	cargo fmt

format-check:
	$(info ****************** running rustfmt in check mode ******************)
	cargo fmt --check