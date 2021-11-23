
dryrun_local=cargo run -- dry
#dryrun_local=RUST_BACKTRACE=full cargo run --bin dryrun -- --debug
dryrun_installed=noname dry
dryrun=${dryrun_local}
default: test

test:  lint
	RUST_BACKTRACE=1 RUST_LOG=debug cargo test --verbose

lint:
	cargo clippy

format: 
	cargo fmt

build_verbose:
	cargo build --verbose

build:
	cargo check 
	cargo build

clean:
	cargo clean

update:
	cargo build

install:
	cargo install

d:
	./demo.sh


##### test groups ####
interactive: interactive x_interactive

tests: passive active x x_active active_env xvar a a2 i noargs data stdout

broken: f t_mkdir 

notyet: unapply cmd apply_interactive apply_passive apply_interactive

errs: err_no_command err_notset er_invalid_command err_novar err_noval err_t_deny

##### tests ####

noargs: 
	$(dryrun) 
help:
	$(dryrun) --help

cmd:
	${dryrun} v mode=600 if Makefile of /tmp/ cp %%if%% %%of%%

unapply:
	cargo run -- unapply example1 

apply_with_var: 
	cargo run -- apply --ifnot 'test -f myfile.config' --then 'echo key=value > myfile.config'
	diff <(echo '') myfile.config

apply_passive: 
	rm -v myfile.config
	cargo run -- apply --passive --ifnot 'test -f myfile.config' --then 'echo key=value > myfile.config'
	diff <(echo '') myfile.config
apply_active: 
	rm -vf myfile.config
	cargo run --active apply --interactive --ifnot 'test -f myfile.config' --then 'echo key=value > myfile.config'
	diff <(echo 'key=value') myfile.config
apply_interactive: 
	rm -vf myfile.config
	cargo run -- apply --interactive --ifnot 'test -f myfile.config' --then 'echo key=value > myfile.config'
	diff <(echo 'key=value') myfile.config

a2: 
	cargo run -- apply --ifnot 'test -f foo' --then 'touch foo'
a:
	cargo run -- apply example1 

i:
	cargo run -- is_applied example1 

fix:
	cargo fix

t_mkdir:
	$(dryrun) --active t <(echo foo) /tmp/foo/deleteme


err_no_command: 
	$(dryrun) x lls -l || true
err_notset:
	$(dryrun) --active v no_value fake_value t 'data:key1 is @@key1@@' $@.out||true
er_invalid_command:
	${dryrun} foo ||true
err_novar:
	${dryrun} v||true
err_noval:
	${dryrun} v x||true
err_t_deny_mkdir:
	$(dryrun) t <(echo foo) /root/foo/deleteme || true


f:
	$(dryrun) v key1 val1 f 'data:key1 is @@key1@@' 'data:key1 is @@key1@@' template/upper.out /usr/bin/tr 'a-z' 'A-Z'
	$(dryrun) --active v key1 val1 f 'data:key1 is @@key1@@' template/upper.out /usr/bin/tr 'a-z' 'A-Z'
passive:
	$(dryrun) --active v key1 fake_value t 'data:key1 is @@key1@@' $@.out
	$(dryrun) v key1 real_value t 'data:key1 is @@key1@@' $@.out
	diff -ZB $@.out <(echo key1 is fake_value)
active:
	$(dryrun) --active v key1 fake_value t 'data:key1 is @@key1@@' $@.out
	$(dryrun) --active v key1 real_value t 'data:key1 is @@key1@@' $@.out
	diff -ZB $@.out <(echo key1 is real_value)
interactive:
	$(dryrun) --active v key1 fake_value t 'data:key1 is @@key1@@' $@.out
	$(dryrun) --interactive v key1 real_value t 'data:key1 is @@key1@@' $@.out

x:
	$(dryrun) --active v key1 fake_value t 'data:key1 is @@key1@@' $@.out
	$(dryrun) x chmod 600 $@.out
	diff -ZB $@.out <(echo key1 is fake_value )
x_active:
	$(dryrun) --active v key1 fake_value t 'data:key1 is @@key1@@' $@.out
	$(dryrun) --active x chmod 600 $@.out

active_env: DRYRUN_ACTIVE=1
active_env:
	$(dryrun) x ls -l Makefile

x_interactive:
	$(dryrun) --active v key1 fake_value t 'data:key1 is @@key1@@' $@.out
	$(dryrun) --interactive x chmod 600 $@.out
xvar:
	$(dryrun) v f $@.out x chmod 600 @@f@@

cleantmp:
		rm *.tmp *.tmp.sh *.out

data:
	cargo run -- dry --active v var Hello t data:=@@var@@= out

stdout:
	cargo run -- dry --active v var Hello t data:=@@var@@= /dev/stdout
