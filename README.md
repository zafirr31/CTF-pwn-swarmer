# CTF pwn swarmer 

A tool to spawn multiple instances of your pwn exploit to run concurrently. Useful for cases where the pwn challenge requires brute force.

## How to use

Add `if args.SWARM` cases to your exploit for lines that is only intended to be run when using this tool. For example, I use the following lines in my exploit script:

```py
if not args.SWARM:
    p.interactive()
else:
    # print out the flag to stdout
    p.sendline("cat /flag*")
    p.sendline("cat /home/*/flag*")
    print(p.recvall(timeout=3), flush=True)
```

Run the tool like so:
```
ctf_pwn_swarmer ./exploit.py --flag-format=FLAG --num-processes=3
```

The tool will automatically stop when a line printed by your script contains the flag format

## Installation

Download the pre-compiled binary and add it to any folder in `PATH`

You can also compile the binary yourself using cargo
