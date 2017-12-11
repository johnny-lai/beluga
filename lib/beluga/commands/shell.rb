require_relative 'base.rb'

module Beluga
  module Commands
    class Shell < Base
      def initialize(app, options = {})
        @exe = options["command"]
        @environ = options["environment"] || {}
        @extra_hosts = options["extra_hosts"] || []
        super
      end
      
      def cmdline(args)
        @exe % args.join(" ")
      end
      
      def environ
        @environ
      end
      
      def extra_hosts
        @extra_hosts
      end
      
      def options
        {
          command: @exe,
          environment: @environ,
          extra_hosts: @extra_hosts
        }
      end
    end
  end
end
