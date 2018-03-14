# Beluga

[![Build Status](https://travis-ci.org/johnny-lai/beluga.svg?branch=master)](https://travis-ci.org/johnny-lai/beluga)

Beluga is a tool for generating base docker images capable of running your Rails application. Since all Ruby applications require Ruby, and the gems in their Gemfile.lock, essentially the way the base image share similarities.

It currently assumes a Rails application, with a package.json file.

It uses the MIT-LICENSE.

## Installation

1. Clone the repo

2. Build
  ```
  $ make
  ```

## Configuration

Add new commands and configure images using config/beluga.yml. If you provide configuration
for existing commands/images, the settings will be merged with beluga's config/default.yml.

```
app:
  version: -- A value that you can use to force modify the digest
images:
  <image-name>:
    tag: -- Docker Label. %s is the digest
    id_rsa: -- Where to find your id_rsa file. Defaults to ~/.ssh/id_rsa>
    from: -- From image name. Example 'devbase'
    extra_packages: -- List of extra packages
    extra_build_instructions: -- Extra Dockerfile instructions that are added to end
commands:
  <command-name>:
    command: -- Command to run. %s is a replacement for all the arguments.
    image: -- Name of docker image to run command in
    extra_hosts:
      - -- Extra hosts. For example: "db:<%= host_public_ip %>"
```
The configuration file is ERB aware, and has the following extra functions:

* `host_public_ip`: Returns the first public IP of the host

## Setting up database access on the Mac

By default, beluga uses UNIX sockets to connect the database on your host with the code running in docker. Unfortunately, docker does not support UNIX sockets on the Mac.

For the Mac, instead, you can connect via the `db` host and port 3306.

* Configure your MySQL server to serve on the public IP

* Add `db` to `/etc/hosts`. Within the container, `db` will refer to the host. Outside it will refer to localhost
```
127.0.0.1        db
```

* Create a user and password on MySQL
```
$ mysql -u root -p
> CREATE USER 'user'@'%' IDENTIFIED BY 'apasswordforamysqluser';
> GRANT ALL PRIVILEGES ON *.* TO 'user'@'%';
```

* In your `config/database.yml`, you would then use:
```
development:
  # ...
  host: db
  username: user
  password: apasswordforamysqluser
```

After this, the same config/database.yml should be usable both within docker, and on the host. This works because the default configuration adds a `db` host that points to the host's first *public* address. See https://github.com/johnny-lai/beluga/blob/master/config/default.yml#L14.

## Examples

All examples should be run from your rails application root folder.

### Build the devbase image

```
$ beluga image build
```

### Build and deploy the devbase image

```
$ beluga image build push
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

### Build the testbase image

```
$ beluga -i testbase image build
```

### Start bash in the testbase image

```
$ beluga -i testbase exec bash
```
