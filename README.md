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
```

### Run rspec in devbase

Run the following from your rails application root folder.
```
$ beluga rspec spec/your_test
```
