all: deps
	cargo build --release
	make clean

deps: osdeps rust nanomsg

rust:
	curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly

nanomsg:
	wget https://github.com/nanomsg/nanomsg/archive/1.1.4.tar.gz
	tar -xvzf 1.1.4.tar.gz
	cd nanomsg-1.1.4 \
		&& mkdir build \
		&& cd build \
		&& cmake .. \
		&& cmake --build . \
		&& sudo cmake --build . --target install
	sudo ldconfig

osdeps:
	sudo apt-get install -y libxtst-dev curl pkg-config cmake

clean:
	rm -rf nanomsg-1.1.4
	rm 1.1.4.tar.gz

install:
	sudo cp target/release/xrecord-echo /usr/local/bin
	sudo cp 50-systemd-user.sh /etc/X11/xinit/xinitrc.d
	mkdir ~/.config/systemd/user --parents --verbose
	cp --verbose xrecord-echo.service ~/.config/systemd/user
	systemctl --user enable xrecord-echo.service
	systemctl --user start xrecord-echo.service

.PHONY: clean deps rust nanomsg
