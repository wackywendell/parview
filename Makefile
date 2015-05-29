lib/x86_64-unknown-linux-gnu/libglfw3.so:
	mkdir -p lib/x86_64-unknown-linux-gnu
	ln -s /usr/lib/libglfw.so.3.1 lib/x86_64-unknown-linux-gnu/libglfw3.so

link: lib/x86_64-unknown-linux-gnu/libglfw3.so
