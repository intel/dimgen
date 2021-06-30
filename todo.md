## todo

- list how many package sources are going to be fetched and show a delta of pkgs that
couldn't be fetched at the end
- ensure that packages are downloaded before creating compressing files
- add a timeout - if the package downloads dont start, exit after timeout
- if docker is not there, try podman
- change commands to http or unix url call of docker daemon
- dont create sources if already available
- dont use the OS tools to fetch the sources, check source code for yum utils and apt get source and code it out 
- add source repos for ubuntu 18.04
