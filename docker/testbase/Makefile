include ../make/env.mk

DOCKERFILE = $(BUILD_ROOT)/testbase.dockerfile

BUILD_SRC = $(SRCROOT)/dockerfile.erb

build: $(BUILD_SRC)
	beluga -a $(RAILS_ROOT) -i testbase image dockerfile > $(DOCKERFILE)
	$(DOCKER) build -f $(DOCKERFILE) -t $(APP_DOCKER_LABEL) $(BUILD_ROOT)

push:
	$(DOCKER) push $(APP_DOCKER_LABEL)

pull:
	$(DOCKER) pull $(APP_DOCKER_LABEL)

clean:
	rm -rf $(BUILD_ROOT)/*

.PHONY: build push
