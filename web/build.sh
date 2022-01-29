#!/bin/bash

cd $(dirname $0)

run_in_docker() {
	if tty -s; then
		local tty_flag="--tty"
	else
		local tty_flag=""
	fi

	docker run \
		--rm \
		--interactive \
		${tty_flag} \
		--volume $(pwd):/work \
		--workdir /work \
		planc-frontend-build \
		$@
}

docker build \
	--tag planc-frontend-build \
        --pull \
        --build-arg=uid=$(id -u) \
        --build-arg=gid=$(id -g) \
        .

run_in_docker npm install
run_in_docker npm run build
