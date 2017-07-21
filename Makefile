all: deps
	cargo build --release
	make clean

deps: osdeps rust nanomsg

rust:
	curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly

nanomsg:
	wget https://github.com/nanomsg/nanomsg/archive/1.0.0.tar.gz
	tar -xvzf 1.0.0.tar.gz
	cd nanomsg-1.0.0 \
		&& mkdir build \
		&& cd build \
		&& cmake .. \
		&& cmake --build . \
		&& sudo cmake --build . --target install
	sudo ldconfig

osdeps:
	sudo apt-get install libxtst-dev curl pkg-config

clean:
	rm -rf nanomsg-1.0.0
	rm 1.0.0.tar.gz

install:
	sudo cp target/release/xrecord-echo /usr/local/bin
	mkdir ~/.config/systemd/user --parents --verbose
	cp --verbose xrecord-echo.service ~/.config/systemd/user
	systemctl --user enable xrecord-echo.service
	systemctl --user start xrecord-echo.service

.PHONY: clean deps rust nanomsg
