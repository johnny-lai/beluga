require 'rake'

module Beluga
  module Images
    class Base
      include FileUtils
      
      attr_accessor :app, :extra_packages
      
      def initialize(app, options = {})
        @app = app
        @tag = options["tag"]
        @id_rsa = options["id_rsa"] || "~/.ssh/id_rsa"
        @extra_packages = options["extra_packages"]
      end
      
      def exe
        @exe ||= ENV["DOCKER"] || "docker"
      end

      def image
        @tag % app.digest
      end

      def options
        {
          tag: @tag,
          id_rsa: @id_rsa,
          extra_packages: @extra_packages
        }
      end

      def src_root_d
        "/app"
      end

      def default_opts
        return @default_opts if @default_opts
        
        opts = <<~eos.tr!("\n", " ")
          #{ENV["DOCKER_EXTRA_OPTS"]}
          -v #{app.root}:#{src_root_d}
          -w #{src_root_d}
          -e IN_DOCKER=true
          -e DEV_UID=#{Process.uid}
          -e DEV_GID=#{Process.gid}
          --net=bridge
        eos
        opts << " -it" if $stdout.isatty
        opts << " -v #{app.db_socket}:#{app.db_socket}" if app.db_socket
        puts opts
        @default_opts = opts
      end

      def run(c, args, extra_opts = "")
        opts = ""
        
        c.environ.each do |k, v|
          opts << " -e #{k}=#{v}"
        end
        
        c.extra_hosts.each do |v|
          opts << " --add-host=#{v}"
        end
        
        sh "#{exe} run --rm #{default_opts} #{opts} #{extra_opts} #{image} #{c.cmdline(args)}"
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
        "RAILS_ROOT=#{app.root} " +
        "APP_DOCKER_LABEL=#{image} " +
        "DIGEST=#{app.digest} " +
        "ID_RSA=#{@id_rsa} " +
        "EXTRA_PACKAGES=\"#{extra_packages.join(' ')}\""
      end
      
      def make(command)
        FileUtils.cd build_root do
          sh "#{environment} make #{command}"
        end
      end
    end
  end
end
