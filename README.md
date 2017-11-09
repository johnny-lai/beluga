# Beluga

[![Build Status](https://travis-ci.org/johnny-lai/beluga.svg?branch=master)](https://travis-ci.org/johnny-lai/beluga)

This project rocks and uses MIT-LICENSE.

## Installation

1. Clone the repo

2. Build gem and install locally
  ```
  $ rake install:local
  ```

## Configuration

Add new commands and configure images using config/beluga.yml

## Limitations

* Only support database connections using sockets. This does not work on MacOS.

## Examples

### Build the devbase image

Run the following from your rails application root folder.
```
$ beluga images build
```

### Run your command in devbase

Run the following from your rails application root folder.
```
$ beluga exec echo "hello from devbase"
docker run --rm  -v ...:/app -v /private/tmp/mysql.sock:/private/tmp/mysql.sock -w /app -e IN_DOCKER=true -e DEV_UID=501 -e DEV_GID=20 --net=bridge   -it beluga-devbase:41a0ab20068ceedb2b15ccfa8ad1931bb9d50a4a echo hello from devbase
groupadd: GID '20' already exists
Adding user `dev' ...
Adding new user `dev' (501) with group `dialout' ...
Creating home directory `/home/dev' ...
Copying files from `/etc/skel' ...
Adding user `dev' to group `sudo' ...
Adding user dev to group sudo
Done.
hello from devbase
```

### Run rspec in devbase

Run the following from your rails application root folder.
```
$ beluga rspec spec/your_test
```
