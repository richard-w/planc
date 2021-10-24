#!/bin/bash

cd $(dirname $0)

docker build \
        --tag planc-frontend-build \
        --pull \
        --build-arg=uid=$(id -u) \
        --build-arg=gid=$(id -g) \
        .

if tty -s; then
	tty_flag="--tty"
else
	tty_flag=""
fi

docker run \
	--rm \
	--interactive \
	${tty_flag} \
	--volume $(pwd):/work \
	--workdir /work \
	planc-frontend-build \
	ng build
