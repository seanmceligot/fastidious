
dryrun_local=cargo run -- dry
#dryrun_local=RUST_BACKTRACE=full cargo run --bin dryrun -- --debug
dryrun_installed=noname dry
dryrun=${dryrun_local}
default: test

a:
	cargo run -- apply example1 

i:
	cargo run -- is-applied example1 

fix:
	cargo fix

t_mkdir:
	$(dryrun) t dryrun.sh /tmp/deleteme

errs: err_no_command err_notset er_invalid_command err_novar err_noval err_t_deny

err_no_command: 
	$(dryrun) -- x lls -l || true
err_notset:
	$(dryrun) --active v no_value fake_value t template/test.config template/out.config||true
er_invalid_command:
	${dryrun} foo ||true
err_novar:
	${dryrun} v||true
err_noval:
	${dryrun} v x||true
err_t_deny_mkdir:
	$(dryrun) t dryrun.sh /root/foo/deleteme || true

f:
	$(dryrun) v key1 val1 f template/test.config template/upper.out /usr/bin/tr 'a-z' 'A-Z'
	$(dryrun) --active v key1 val1 f template/test.config template/upper.out /usr/bin/tr 'a-z' 'A-Z'
passive:
	$(dryrun) --active v value fake_value t template/test.config template/out.config
	$(dryrun) v value real_value t template/test.config template/out.config
active:
	$(dryrun) --active v value fake_value t template/test.config template/out.config
	$(dryrun) --active v value real_value t template/test.config template/out.config
interactive:
	$(dryrun) --active v value fake_value t template/test.config template/out.config
	$(dryrun) --interactive v value real_value t template/test.config template/out.config

x:
	$(dryrun) --active v value fake_value t template/test.config template/out.config
	$(dryrun) x chmod 600 template/out.config
x_active:
	$(dryrun) --active v value fake_value t template/test.config template/out.config
	$(dryrun) --active x chmod 600 template/out.config

active_env: DRYRUN_ACTIVE=1
active_env:
	$(dryrun) "--" x ls -l $(MAKE)

x_interactive:
	$(dryrun) --active v value fake_value t template/test.config template/out.config
	$(dryrun) --interactive x chmod 600 template/out.config
xvar:
	$(dryrun) v f template/out.config x chmod 600 @@f@@
create:
	rm -vf template/out.config
	$(MAKE) active	

test:  lint
	RUST_BACKTRACE=1 cargo test

help:
	$(dryrun) --help

lint:
	cargo clippy

format: 
	cargo fmt

verbose:
	cargo build --verbose

build:
	cargo check 
	cargo build

noargs: 
	echo cp out1/myconfig project/myconfig
	$(dryrun) --debug

cmd:
	echo dryrun v mode=600 if Makefile of /tmp/ cp %%if%% %%of%%

clean:
	cargo clean
	rm -rvf out

update:
	cargo build

install:
	cargo install

d:
	./demo.sh

interactive: interactive x_interactive
tests: passive active x x_active active_env xvar cmd 
