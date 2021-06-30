## Docker Image Metadata GENerator

**dimgen** - Get sources of installed packages (using the os package manager) in a docker image.

![Alt text](./svg/dimgen.svg)

### Supported OSes


The tool has be tested with images based on :

- Debian Buster
- Centos
- Fedora
- Redhat
- Redhat UBI
- Ubuntu 20.04


### Prerequistes

Docker has to be installed on the system and preferably docker command should be [sudoless](https://linoxide.com/use-docker-without-sudo-ubuntu/),
otherwise use `sudo` to run the tool.


### build 

Use `cargo` to build the tool:

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

Once sucessfully built, the binaries would be present in the `target` directory.

### command line options

```
Usage: dimgen -i <image> [-s]

generate source container images form binary containers

Options:
  -i, --image       name of the image for which source image has to be generated
  -s, --seperate    generate sources for base image and additional layers
                    seperately
  --help            display usage information
```

### Warning

This solution is in its very early stages, so issues are expected. If you
are using this tool to generate sources for any software release requirements, 
it's your responsibility to ensure that all the sources are indeed downloaded and compressed into the tar file. 

### Examples

```bash
dimgen --image ubuntu:20.04
dimgen --seperate --image sysstacks/dlrs-tensorflow-ubuntu:latest 
dimgen --image ubi:latest 
```

Example 1

Let's try to get the sources for `ubuntu:latest` docker image.

```bash
./dimgen -s -i ubuntu:latest
```

Here, in this command we use `-s` but as the image is itself a base image, the layers
compressed file doesn't container any sources. The output of the command above
looks like this:

```bash
 [1/3] 🚚   setting up container for image :: ubuntu:latest [✔]
 [2/3] 🔍   fetching metadata for container :: 7a6ce9cc87a [✔]
 [*/*]      generating sources separately for base and layers
⠁ [00:00:00]  [▸▸]
 [*/*] 0 sources fetched [✔]
 [*/*] 🚚   compressing source packages [✔]
 [*/*] 92 sources fetched [✔]
 [*/*] 🚚   compressing source packages [✔]
 [*/*] 📃   sources compressed to : collaterals/base_26b77e5843_ubuntu20.04.tar.gz [✔]
 [3/3] ✨   stopping container :: 7a6ce9cc87a [✔]
 ```
 The sources packages gets compressed and saved to `collaterals` directory.

Let's see what the `collaterals` directory contain:

```bash

rahul in oraclebox in dimgen/bin/collaterals on  master [?] on ☁️ rahulunair@gmail.com
❯ ls -lhrt collaterals
.rw-r--r-- rahul rahul  83 B  Wed Apr 14 10:51:03 2021  layers_26b77e5843_ubuntulatest.Dockerfile
.rw-r--r-- rahul rahul 659 B  Wed Apr 14 10:51:03 2021  layers_26b77e5843_ubuntulatest.tar.gz
.rw-r--r-- rahul rahul  80 B  Wed Apr 14 10:51:12 2021  base_26b77e5843_ubuntu20.04.Dockerfile
.rw-r--r-- rahul rahul 238 MB Wed Apr 14 10:51:13 2021  base_26b77e5843_ubuntu20.04.tar.gz
```

As seen, the base image sources file got generated along with a Dockerfile template. This dockerfile
is an example that be referred to create source images.


### Contributions

Contributions are welcome, this is a alpha stage project and has a lot of rough edges,
there is a lot of areas for contributions, please see the `todo.md` to get ideas :).

### Security Issues

Security issues can be reported to Intel's security incident response team via https://intel.com/security.

### Release Notes

--------------------------------------------------------------------------------
#### v0.1.0

First release of `dimgen`.

###### issues fixed

- fixes for compression bug
- added support for debain buster

    **known bugs** - copying sources from a container with multi gig sources
    is slow at this point, a future release will remove copying the sources from
    the container by mounting a volume directly on the host.
--------------------------------------------------------------------------------


