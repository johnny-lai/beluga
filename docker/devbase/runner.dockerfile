ARG from
FROM $from

RUN curl -sL https://deb.nodesource.com/setup_6.x | bash - && apt-get update && apt-get install -y nodejs default-jre mysql-client sudo

WORKDIR /build/
COPY package.json npm-shrinkwrap.json ./
RUN NODE_ENV=production npm install

ARG id_rsa
WORKDIR /build/
COPY Gemfile Gemfile.lock ./
RUN mkdir /root/.ssh && \
    echo "$id_rsa" > /root/.ssh/id_rsa && \
    chmod 600 /root/.ssh/id_rsa && \
    echo "Host github.com\n\tUser git\n\tStrictHostKeyChecking no\n" >> /root/.ssh/config && \
    mkdir -p /usr/local/bundle && \
    bundle install --path /usr/local/bundle && \
    rm -rf /root/.ssh

COPY sudoers /etc/sudoers
COPY entrypoint.sh /entrypoints/devbase.sh

ENV BUNDLE_CACHE_PATH ../usr/local/bundle/cache
ENV BUNDLE_GEMFILE Gemfile
ENTRYPOINT ["/entrypoints/devbase.sh"]

ARG extra_packages
RUN apt-get install -y $extra_packages
