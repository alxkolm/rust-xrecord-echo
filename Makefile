all: deps
	cargo build --release
	make clean

deps: libxtst rust nanomsg

rust:
	curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly

nanomsg:
	wget https://github.com/nanomsg/nanomsg/releases/download/0.6-beta/nanomsg-0.6-beta.tar.gz
	tar -xvzf nanomsg-0.6-beta.tar.gz
	cd nanomsg-0.6-beta && ./configure && make && sudo make install
	sudo ldconfig

libxtst:
	apt-get install libxtst-dev

clean:
	rm -rf nanomsg-0.6-beta
	rm nanomsg-0.6-beta.tar.gz

.PHONY: clean deps rust nanomsg
