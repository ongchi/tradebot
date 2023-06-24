DOCKER_RUN = run --platform linux/x86_64 --rm
DOCKER_IMAGE = rust-cross-build/x86_64-linux
DOCKER_VOL = -v `pwd`:/app
DOCKER_ENV = -e CARGO_HOME=/app/cross_build/cargo_home -e CARGO_TARGET_DIR=/app/cross_build/target

.PHONY: build
build:
	docker ${DOCKER_RUN} ${DOCKER_VOL} ${DOCKER_ENV} ${DOCKER_IMAGE}
