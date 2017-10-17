# These are local paths
SRCROOT ?= $(abspath .)
RAILS_ROOT ?= $(abspath $(SRCROOT)/../..)
BUILD_ROOT ?= $(RAILS_ROOT)/tmp/beluga

DOCKER_ROOT ?= $(SRCROOT)/docker

# Docker commands
DOCKER_VER_NUM ?= $(shell docker --version | cut -f1 "-d," | cut -f3 "-d ")
DOCKER_VER_MAJOR := $(shell echo $(DOCKER_VER_NUM) | cut -f1 -d.)
DOCKER_VER_MINOR := $(shell echo $(DOCKER_VER_NUM) | cut -f2 -d.)
DOCKER_GT_1_12 := $(shell [ $(DOCKER_VER_MAJOR) -gt 1 -o \( $(DOCKER_VER_MAJOR) -eq 1 -a $(DOCKER_VER_MINOR) -ge 12 \) ] && echo true)

ifeq ($(DOCKER_GT_1_12),true)
	DOCKER_OPT_TAG_FORCE=
else
	DOCKER_OPT_TAG_FORCE=-f
endif

DOCKER ?= docker
ifneq ($(findstring gcr.io/,$(APP_DOCKER_LABEL)),)
	DOCKER_PUSH ?= gcloud docker push
else
	DOCKER_PUSH ?= $(DOCKER) push
endif

# Sha
SHA1SUM := $(shell { command -v sha1sum || command -v shasum; } 2>/dev/null)

$(BUILD_ROOT):
	mkdir -p $@
