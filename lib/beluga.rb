require 'beluga/commands'
require 'beluga/configuration'
require 'beluga/images'
require 'beluga/main'
require 'beluga/rails_app'

module Beluga
  def self.root
    @root ||= Gem.loaded_specs['beluga'].full_gem_path
  end
end
