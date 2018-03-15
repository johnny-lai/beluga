pub const DEVBASE: &'static str = "
FROM {{from}}

RUN curl -sL https://deb.nodesource.com/setup_6.x | bash - && apt-get update && apt-get install -y nodejs default-jre mysql-client sudo

RUN touch /etc/sudoers

WORKDIR /entrypoints
RUN {{write_rsc \"entrypoint.sh\" \"devbase.sh\"}}
ENTRYPOINT ['/entrypoints/devbase.sh']

{{build_instructions}}
";

pub const NPM_INSTALL: &'static str = "
WORKDIR /build/
COPY package.json npm-shrinkwrap.json ./
RUN NODE_ENV=production npm install
";

pub const GEM_INSTALL: &'static str = "
WORKDIR /build/
COPY Gemfile Gemfile.lock ./
RUN mkdir /root/.ssh && \\
    echo '{{id_rsa}}' > /root/.ssh/id_rsa && \\
    chmod 600 /root/.ssh/id_rsa && \\
    echo 'Host github.com\\n\\tUser git\\n\\tStrictHostKeyChecking no\\n' >> /root/.ssh/config && \\
    mkdir -p /usr/local/bundle && \\
    bundle install --path /usr/local/bundle && \\
    rm -rf /root/.ssh
ENV BUNDLE_CACHE_PATH ../usr/local/bundle/cache
ENV BUNDLE_GEMFILE Gemfile
";

//- entrypoint_sh --------------------------------------------------------------
pub const ENTRYPOINT_SH: &'static str = "
#!/bin/bash

# Create links
if [ ! -d node_modules ]; then
  ln -s /build/node_modules node_modules
fi

# Set up user
USER=root

if [ $(whoami) == 'root' ] && [ $DEV_UID ]; then
    groupadd --gid $DEV_GID dev
    adduser --disabled-password --gecos '' --uid $DEV_UID --gid $DEV_GID dev
    adduser dev sudo
    echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

    # Fix permissions of home directory
    # Since we mount /home/dev/.kube/config, this may be necessary
    chown -R $DEV_UID /home/dev
    chgrp -R $DEV_GID /home/dev

    USER=dev
fi

if [ '$1' ]; then
    sudo -HEu $USER $@
else
    echo 'starting bash'
    /bin/bash
fi
";
