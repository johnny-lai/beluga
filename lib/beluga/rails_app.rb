require 'digest'
require 'ostruct'
require 'yaml'

module Beluga
  class RailsApp
    attr_accessor :root

    def initialize(root)
      @root = File.expand_path(root)
    end

    def config
      return @config if @config
      
      @config = {}
      ["#{Beluga.root}/config/default.yml", "#{root}/config/beluga.yml"].each do |yml|
        if File.exists?(yml)
          @config.merge!(YAML.load(File.open(yml, "r")))
        end
      end
      
      @config
    end

    def db_socket
      return @db_socket unless @db_socket.nil?

      dbhash = YAML.load(File.open("#{root}/config/database.yml",'r'))
      @db_socket = dbhash["development"]["socket"]
    end
    
    def digest
      return @digest if @digest

      sha1 = ::Digest::SHA1.new
      %w[.ruby-version package.json npm-shrinkwrap.json Gemfile Gemfile.lock].each do |f|
      	sha1 << File.read(File.join(root, f))
      end

      @digest = sha1.hexdigest
    end
    
    def images
      @images ||= {
        "devbase" => Images::Devbase.new(self, config["images"]["devbase"]),
        "testbase" => Images::Testbase.new(self, config["images"]["testbase"]),
      }
    end
    
    def commands
      @commands ||= Hash.new do |h, k|
        h[k] = Commands::Shell.new(self, config["commands"][k])
      end
    end
  end
end
