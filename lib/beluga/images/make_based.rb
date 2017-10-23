require 'erb'
require 'rake'

module Beluga
  module Images
    class MakeBased
      include FileUtils
      
      attr_accessor :app
      
      def initialize(app, options = {})
        @app = app
        @make_root = options[:make_root]
        @options = options[:options] || {}
        @options["id_rsa"] ||= "~/.ssh/id_rsa"
        @options["extra_packages"] ||= []
      end
      
      def exe
        @exe ||= ENV["DOCKER"] || "docker"
      end

      def image
        @options["tag"] % app.digest
      end

      def options
        @options
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
      
      def dockerfile
        from = if @options["from"]
          app.images[@options["from"]].image
        else
          nil
        end
        extra_build_instructions = @options["extra_build_instructions"]

        erb = ERB.new(File.read("#{make_root}/dockerfile.erb"))
        erb.result(binding)
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
      
      def make_root
        @make_root
      end
      
      def environment
        { RAILS_ROOT: app.root,
          BUILD_ROOT: app.build_root,
          APP_DOCKER_LABEL: image,
          DIGEST: app.digest,
          ID_RSA: @options["id_rsa"],
          EXTRA_PACKAGES: @options["extra_packages"].join(' '),
        }
      end
      
      def make(command)
        FileUtils.cd make_root do
          env_opts = environment.map do |k,v|
            "#{k}='#{v}'"
          end.join(" ")
          sh "#{env_opts} make #{command}"
        end
      end
    end
  end
end
