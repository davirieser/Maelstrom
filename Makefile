
TEST_NAME ?= echo
TIME_LIMIT ?= 20 
RATE ?= 100
NODES ?= 25

EXTRA ?= 
OPTIONS=-w $(TEST_NAME) --node-count $(NODES) --time-limit $(TIME_LIMIT) --rate $(RATE) $(EXTRA)

all: build
	./maelstrom/maelstrom test --bin ~/.cargo/target/debug/Maelstrom $(OPTIONS)

serve:
	@ ./maelstrom/maelstrom serve

build: src/* src/**
	@ cargo build

