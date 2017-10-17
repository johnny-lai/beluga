FROM ruby:...

RUN curl -sL https://deb.nodesource.com/setup_6.x | bash - && apt-get update && apt-get install -y nodejs default-jre mysql-client sudo

WORKDIR /build/

COPY package.json npm-shrinkwrap.json ./

RUN NODE_ENV=production npm install

WORKDIR /usr/local/bundle

COPY Gemfile Gemfile.lock ./

COPY vendor/cache cache

ENV BUNDLE_CACHE_PATH cache

RUN bundle install --local --system

COPY sudoers /etc/sudoers

COPY devbase-entrypoint.sh /

ENV BUNDLE_CACHE_PATH ../usr/local/bundle/cache

ENTRYPOINT ["/devbase-entrypoint.sh"]
