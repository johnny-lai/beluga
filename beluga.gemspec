$:.push File.expand_path("../lib", __FILE__)

# Maintain your gem's version:
require "beluga/version"

# Describe your gem and declare its dependencies:
Gem::Specification.new do |s|
  s.name        = "beluga"
  s.version     = Beluga::VERSION
  s.authors     = ["Johnny Lai"]
  s.email       = ["johnny.lai@me.com"]
  s.summary     = "Creating docker images with pre-requisites for your rails app"
  s.license     = "MIT"

  s.files = Dir["{bin,app,config,db,lib}/**/*", "MIT-LICENSE", "Rakefile", "README.rdoc"]
  s.test_files = Dir["test/**/*"]

  s.add_dependency "rake"
  
  s.add_development_dependency "minitest"
end
