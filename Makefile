.PHONY=list flash

list:
	@echo flash

flash: target/avr-atmega328p/debug/tea-whistle.elf
	cargo build
	./uno-runner.sh $<

