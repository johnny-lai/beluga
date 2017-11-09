require_relative 'base.rb'

module Beluga
  module Commands
    class Shell < Base
      def initialize(app, options = {})
        @exe = options["exe"]
        @environ = options["environ"] || {}
        super
      end
      
      def cmdline(args)
        @exe % args.join(" ")
      end
      
      def environ
        @environ
      end
    end
  end
end
