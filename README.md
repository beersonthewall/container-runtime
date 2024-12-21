# A container runtime

Sorta like the generic brands that are just named 'bread', but for containers.


## Testing
The steps:
- Create an OCI Bundle
- Run commands against that bundle

### How To Create an OCI Bundle

```bash
mkdir <img-name>
mkdir <img-name>/rootfs
docker pull <img-name>
docker create --name tmp<img-name> <img-name>
docker export tmp<img-name> | tar -C <img-name>/rootfs -xf -
docker rm tmp<img-name>
cd <img-name>
runc spec
```

This will leave you with a root filesystem and a config.json in the <img-name> directory.

### Container Runtime CLI Usage

```bash
container_runtime create <container-id> ./path-to-bundle
container_runtime start <container-id>
container_runtime kill <container-id> <signal>
container_runtime delete <container-id>
container_runtime state <container-id>
```

Note: Certain operations require root

