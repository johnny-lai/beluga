require 'erb'
require 'socket'

module Beluga
  class Configuration
    def self.load_yml(yml_file)
      if File.exists?(yml_file)
        yml = Configuration.new(File.read(yml_file)).result
        YAML.load(yml)
      end
    end

    def initialize(template)
      @template = template
    end
    
    def result
      ERB.new(@template).result(binding)
    end

    #- ERB functions -----------------------------------------------------------
    def host_public_ip
      @host_public_ip ||= Socket.ip_address_list.detect{|intf| intf.ipv4_private?}&.ip_address
    end
  end
end
