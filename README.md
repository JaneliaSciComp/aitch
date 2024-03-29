`aitch` (pronounced like the letter H) implements a light-weight cluster-style
scheduler for use on a single workstation.  Unlike `parallel`, `xargs`, and
other similar utilities, which presume that all jobs use the same amount
of resources, `aitch` permits one to request a different amount from an
arbitrary number of unique kinds of resources.  In effect it implements a
counting semaphore for each resource and provides a means to adjust their
values by non-negative integers.

# Installation #

Install rust as described [here](https://www.rust-lang.org/).  Build with
`cargo build`.

# Basic Usage #

Say for example your machine has six cores and you want to batch a heterogeneous
collection of jobs which require different numbers of cores:

```
hstart 6
hsubmit 2 my-two-slot-job
hsubmit 4 my-four-slot-job [and-its-args...]
hsubmit 1 my-single-slot-job
```

In this case, the third job is queued and remains pending until one of the
first two finish.

Other kinds of resources you might want to meter out include accelerator
cards, memory, and file or network I/O.  So for example, if in addition
your machine also has two GPUs and 32 GB of RAM, the scheduler could be
configured as follows:

```
hstart 6,2,32
hsubmit 1,0,32 my-high-memory-app
hsubmit 2,1,8 my-visualization-app
hsubmit 6,0,1 my-compute-intensive-app
hsubmit 0,0,0 'touch helloworld'
```

Should your jobs need to know which specific slot of a resource they consume,
an environment variable prefixed with "QUEUE" is defined:

```
echo "export CUDA_VISIBLE_DEVICES=$QUEUE1; \
      python -m 'import tensorflow...'" > my-deeplearning-app
hsubmit 2,1,4 my-deeplearning-app
```

In this case, QUEUE1 would be set to either 0 or 1, assuming the `hstart`
from above was still in effect.  QUEUE0 would similarly be two integers
between 0 and 5 separated by a comma.

One can also specify job dependencies:

```
dep1=`hsubmit 6,0,12 do-me-first`
hsubmit 1,0,1 --dep $dep1 wait-for-do-me-first-to-finish
```

and pass in environment variables:

```
hsubmit 1,0,0 --env FOO=bar winnie-the-pooh
```

and redirect the standard streams:

```
hsubmit 1,0,3 --out stdout.txt -err stderr.txt log-the-results
```

Besides the `hstart` and `hsubmit` commands, there are also `hjobs`, `hkill`,
`hnslots`, `hstatus`, and `hstop`.  Usage information for each is displayed
with the `--help` flag.

# Development #

Run the tests with `cargo test`.

Build and test on a local disk with the `--target-dir <DIR>` flag.  Useful if
the source is on a networked fileshare which doesn't support locking.

Update rust and cargo with `rustup update`.

The conda-forge feedstock is at https://github.com/conda-forge/aitch-feedstock.

Check the formatting of the feedstock recipe with `conda smithy recipe-lint`.

Generate a new key for the source code archive with `openssl sha256 <github-tar-gz-file>`.
