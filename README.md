# sps
Some Package Manager

## Demo
You need rust installed. It's available [here](https://www.rust-lang.org/tools/install)

clone the repo

`
$ git clone https://github.com/SamHSmith/sps.git
$ cd sps
`

Build the SPU package utility :

`
$ cd spu && cargo build && cd ..
`

Install the test package :

`
$ sudo ./_spu install testing/hello-1.0-0.sbp.tar.xz
`

Use the package :

`
$ hello`

Upgrade the test package : 

`
$ sudo ./_spu install testing/hello-1.1-0.sbp.tar.xz
`

Downgrade the package :

`
$ sudo ./_spu install testing/hello-1.0-0.sbp.tar.xz
`


Remove the package :

`
$ sudo ./_spu remove hello-1.0-0
`
