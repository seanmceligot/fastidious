![Build Status](https://github.com/seanmceligot/fastidious/actions/workflows/rust.yml/badge.svg)

# build

# use
Test and apply if not already applied

```console
fastidious apply --interactive --ifnot 'test -f hello.sh' --then 'echo -e "#!/bin/sh\necho hello"> hello.sh'
test -f hello.sh
Unapplied
run (y/n): #! /bin/sh
echo -e "#!/bin/sh\necho hello"> hello.sh
y

echo -e "#!/bin/sh\necho hello"> hello.sh
status code:  0
Applied
```

Make executable if not already

```console
fastidious apply --interactive --ifnot 'test -x hello.sh' --then 'chmod 755 hello.sh'
LIVE: run  #! /bin/sh
test -x hello.sh
Unapplied

run (y/n): #! /bin/sh
chmod 755 hello.sh
y

LIVE: run  #! /bin/sh
chmod 755 hello.sh
status code:  0
Applied
```

Does nothing because hello.sh already exists

```console
fastidious apply --interactive --ifnot 'test -f hello.sh' --then 'echo -e "#!/bin/sh\necho hello"> hello.sh'
LIVE: run  #! /bin/sh
test -f hello.sh
status code:  0
Applied
```

Ask before running

```console
fastidious dryrun --interactive x ./hello.sh
run (y/n): "/home/sean/git/rust/fastidous/./hello.sh"
y
LIVE: run  "/home/sean/git/rust/fastidous/./hello.sh"
hello
status code:  0
Script done.
```

Templates

```console
fastidious dryrun --active v key1 real_value t 'data:key1 is @@key1@@' file.out
LIVE: create from template InMemory("key1 is @@key1@@") [1414268916.gen.tmp]  ->file.out
```

Arguments
=========

- --interactive : ask before executing command
- --passive : check permissions and print what would be run
- --active : run without asking
- apply --ifnot <script> --then <script>
- is-applied <script>
- x cmd arg...: run command
- var key value : set variable
