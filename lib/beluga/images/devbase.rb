require_relative 'base.rb'

module Beluga
  module Images
    class Devbase < Base
      def build_root
        "#{Beluga.root}/docker/devbase"
      end
    end
  end
end
