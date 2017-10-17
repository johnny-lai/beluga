include ../make/env.mk

RUBY_VERSION=$(shell cat $(RAILS_ROOT)/.ruby-version)

DIGEST?=$(shell make digest | tail -1)
PREREQUISITES_DIGEST_CACHE=$(BUILD_ROOT)/digest
BUNDLE_CACHE=$(BUILD_ROOT)/bundle-$(DIGEST)

# Files used to calculate the digest
PREREQUISITES_DIGEST_SRC=$(BUILD_ROOT)/.ruby-version \
                         $(BUILD_ROOT)/package.json \
                         $(BUILD_ROOT)/npm-shrinkwrap.json \
                         $(BUILD_ROOT)/Gemfile \
                         $(BUILD_ROOT)/Gemfile.lock.normalized

BUILD_SRC=$(BUILD_ROOT)/runner.dockerfile\
          $(BUILD_ROOT)/sudoers \
          $(BUILD_ROOT)/entrypoint.sh

digest: $(PREREQUISITES_DIGEST_CACHE)
	@cat $(PREREQUISITES_DIGEST_CACHE)

build: $(PREREQUISITES) $(BUILD_SRC)
	if [ ! -f $(BUNDLE_CACHE) ]; then \
		cd $(BUILD_ROOT) && rm -rf vendor/cache && rvm $(RUBY_VERSION) exec bundle package --all; \
		touch $(BUNDLE_CACHE); \
	fi; \
	$(DOCKER) build -f $(BUILD_ROOT)/runner.dockerfile -t $(APP_DOCKER_LABEL) $(BUILD_ROOT); \

push:
	$(DOCKER) push $(APP_DOCKER_LABEL)

pull:
	$(DOCKER) pull $(APP_DOCKER_LABEL)

clean: prerequisites.clean
	rm -rf $(BUILD_ROOT)/*

prerequisites.build: $(DIGEST_PREREQS) $(BUILD_ROOT)/Gemfile.lock

prerequisites.clean:
	rm -f $(PREREQUISITES_DIGEST_SRC)

prerequisites.digest: $(PREREQUISITES_DIGEST_CACHE)

$(PREREQUISITES_DIGEST_CACHE): $(PREREQUISITES_DIGEST_SRC)
	@cat $(PREREQUISITES_DIGEST_SRC) | $(SHA1SUM) | awk '{print $$1}' > $@

$(BUILD_ROOT)/runner.dockerfile: $(SRCROOT)/runner.dockerfile
	cat $< | sed -E -e "s/^FROM ruby:.*$$/FROM ruby:$(RUBY_VERSION)/" > $@

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
