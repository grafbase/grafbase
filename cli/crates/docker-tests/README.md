On MacOS with Colima you'll need to setup the following env vars for the test:

```bash
# our docker client doesn't use docker context to detect colima
export DOCKER_HOST="unix://$HOME/.colima/default/docker.sock"
# Colima only mounts $HOME & /tmp/colima inside the VM
export TMPDIR=/tmp/colima
cargo nextest run -p grafbase-docker-tests
```
