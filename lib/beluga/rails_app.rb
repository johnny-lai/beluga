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
      ["#{Beluga.root}/config/default.yml", "#{root}/config/beluga.yml"].each do |yml_file|
        if File.exists?(yml_file)
          yml = Configuration.new(File.read(yml_file)).result
          @config.merge!(YAML.load(yml))
        end
      end
      
      @config
    end

    def db_socket
      return @db_socket unless @db_socket.nil?

      dbhash = YAML.load(ERB.new(File.read("#{root}/config/database.yml")).result)
      @db_socket = dbhash["development"]["socket"]
    end
    
    def digest
      return @digest if @digest

      sha1 = ::Digest::SHA1.new
      sha1 << version.to_s unless version.nil?
      %w[.ruby-version package.json npm-shrinkwrap.json Gemfile Gemfile.lock].each do |f|
      	sha1 << File.read(File.join(root, f))
      end

      @digest = sha1.hexdigest
    end
    
    def version
      @version ||= config["version"]
    end

    def images
      imgs = Hash.new do |h, k|
        raise "Unknown image: #{k}"
      end
  
      config["images"].each do |k, v|
        imgs[k] = case k
        when "devbase"
          Images::Devbase.new(self, config["images"]["devbase"])
        when "testbase"
          Images::Testbase.new(self, config["images"]["testbase"])
        end
      end

      @images = imgs
    end
    
    def commands
      return @commands if @commands
      
      cmds = Hash.new do |h, k|
        raise "Unknown command: #{k}"
      end
      
      config["commands"].each do |k, v|
        cmds[k] = Commands::Shell.new(self, v)
      end
      
      @commands = cmds
    end
  end
end
