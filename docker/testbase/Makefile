include ../make/env.mk

BUILD_SRC=$(SRCROOT)/dockerfile.erb

build: $(BUILD_SRC)
	erb $(SRCROOT)/dockerfile.erb > $(BUILD_ROOT)/testbase.dockerfile
	$(DOCKER) build -f $(BUILD_ROOT)/testbase.dockerfile -t $(APP_DOCKER_LABEL) $(BUILD_ROOT); \

push:
	$(DOCKER) push $(APP_DOCKER_LABEL)

pull:
	$(DOCKER) pull $(APP_DOCKER_LABEL)

clean:
	rm -rf $(BUILD_ROOT)/*

.PHONY: build push
