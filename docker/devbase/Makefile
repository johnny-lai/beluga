include ../make/env.mk

ID_RSA ?= ~/.ssh/id_rsa
RUBY_VERSION = $(shell cat $(RAILS_ROOT)/.ruby-version)
DOCKERFILE = $(BUILD_ROOT)/runner.dockerfile

BUILD_SRC = $(BUILD_ROOT) \
            $(BUILD_ROOT)/package.json \
            $(BUILD_ROOT)/npm-shrinkwrap.json \
            $(BUILD_ROOT)/Gemfile \
            $(BUILD_ROOT)/Gemfile.lock \
            $(BUILD_ROOT)/sudoers \
            $(BUILD_ROOT)/entrypoint.sh

build: $(PREREQUISITES) $(BUILD_SRC)
	beluga -a $(RAILS_ROOT) -i devbase image dockerfile > $(DOCKERFILE)
	$(DOCKER) build -f $(DOCKERFILE) -t $(APP_DOCKER_LABEL) \
	  --build-arg from="ruby:$(RUBY_VERSION)" \
	  --build-arg id_rsa="$$(cat $(ID_RSA))" \
	  --build-arg extra_packages="$(EXTRA_PACKAGES)" \
	  $(BUILD_ROOT);

push:
	$(DOCKER) push $(APP_DOCKER_LABEL)

pull:
	$(DOCKER) pull $(APP_DOCKER_LABEL)

clean:
	rm -rf $(BUILD_ROOT)/*

$(BUILD_ROOT)/Gemfile: $(RAILS_ROOT)/Gemfile
	cat $< | sed -E -e "s/^.*gem.*(path:|:path) .*$$//" > $@

$(BUILD_ROOT)/Gemfile.lock.normalized: $(RAILS_ROOT)/Gemfile.lock
	cp $< $@
	cp $< $(BUILD_ROOT)/Gemfile.lock
	cd $(BUILD_ROOT) && rvm $(RUBY_VERSION) exec bundle install \
		&& cat Gemfile.lock.normalized | awk "/BUNDLED WITH/{exit} {print}" > t \
		&& mv t Gemfile.lock.normalized

$(BUILD_ROOT)/sudoers: $(SRCROOT)/build/sudoers
	cp $< $@

$(BUILD_ROOT)/entrypoint.sh: $(SRCROOT)/build/entrypoint.sh
	cp $< $@

$(BUILD_ROOT)/%: $(RAILS_ROOT)/%
	cp $< $@

.PHONY: build push
