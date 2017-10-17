require 'rake'

module Beluga
  module Images
    class Base
      include FileUtils
      
      attr_accessor :app
      
      def initialize(app, options = {})
        @app = app
        @tag = options["tag"]
      end
      
      def exe
        @exe ||= ENV["DOCKER"] || "docker"
      end

      def image
        @tag % app.digest
      end
      
      def src_root_d
        "/app"
      end

      def opts
        @opts ||= <<~eos.tr!("\n", " ")
          #{ENV["DOCKER_EXTRA_OPTS"]}
          -v #{app.root}:#{src_root_d}
          -v #{app.db_socket}:#{app.db_socket}
          -w #{src_root_d}
          -e IN_DOCKER=true
          -e DEV_UID=#{Process.uid}
          -e DEV_GID=#{Process.gid}
          --net=bridge
        eos
      end

      def run(c, args, extra_opts = "")
        env_opts = c.environ.map do |k, v|
          "-e #{k}=#{v}"
        end.join(" ")
        
        sh "#{exe} run --rm #{opts} #{env_opts} #{extra_opts} #{image} #{c.cmdline(args)}"
      end
      
      #- Commands -----------------------------------------------------------------------
      def build
        make("build")
      end
      
      def clean
        make("clean")
      end
      
      def push
        make("push")
      end
      
      def pull
        make("pull")
      end
      
      protected
      
      def build_root
        raise NotImplementedError, "build_root should be defined"
      end
      
      def environment
        "RAILS_ROOT=#{app.root} APP_DOCKER_LABEL=#{image} DIGEST=#{app.digest} "
      end
      
      def make(command)
        FileUtils.cd build_root do
          sh "#{environment} make #{command}"
        end
      end
    end
  end
end
