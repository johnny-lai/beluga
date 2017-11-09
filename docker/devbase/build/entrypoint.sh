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

if [ "$1" ]; then
	sudo -HEu $USER $@
else
	echo "starting bash"
	/bin/bash
fi
