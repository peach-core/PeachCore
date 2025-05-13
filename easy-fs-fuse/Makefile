.PHONY: build clean

build: 
	@mkdir image -p
	@cargo run --release -- -s ./elf/ -t ./elf/

clean: 
	@cargo clean
	rm ./elf ./image/fs.img -r 2>/dev/null