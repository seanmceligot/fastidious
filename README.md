```
fastidious apply --interactive --ifnot 'test -f hello.sh' --then 'echo -e "#!/bin/sh\necho hello"> hello.sh'
test -f hello.sh
Unapplied
run (y/n): #! /bin/sh
echo -e "#!/bin/sh\necho hello"> hello.sh
y

echo -e "#!/bin/sh\necho hello"> hello.sh
status code:  0
Applied


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

fastidious apply --interactive --ifnot 'test -f hello.sh' --then 'echo -e "#!/bin/sh\necho hello"> hello.sh'
LIVE: run  #! /bin/sh
test -f hello.sh
status code:  0
Applied


fastidious dryrun --interactive x ./hello.sh
run (y/n): "/home/sean/git/rust/fastidous/./hello.sh"
y
LIVE: run  "/home/sean/git/rust/fastidous/./hello.sh"
hello
status code:  0
Script done.
```
